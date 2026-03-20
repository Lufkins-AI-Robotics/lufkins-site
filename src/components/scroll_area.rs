//! Custom overlay scrollbar wrapper component.
//!
//! [`ScrollArea`] renders its children inside a scrollable viewport with
//! native scrollbars hidden and custom overlay thumb/track elements.
//! The custom scrollbars fade in on hover and support click-drag on
//! the thumb and click-to-jump on the track.

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, HtmlElement, MutationObserver, MutationObserverInit, PointerEvent};
use yew::prelude::*;

/// Scroll direction the component should support.
#[derive(Clone, Copy, PartialEq, Default)]
#[allow(dead_code)]
pub enum ScrollDirection {
    #[default]
    Vertical,
    Horizontal,
    Both,
}

/// Which axis a drag operation is acting on.
#[derive(Clone, Copy)]
enum DragAxis {
    Y,
    X,
}

/// State captured at the start of a thumb drag.
struct DragStart {
    axis: DragAxis,
    pointer_origin: f64,
    scroll_origin: f64,
    scroll_range: f64,
    track_len: f64,
    thumb_len: f64,
}

const MIN_THUMB: f64 = 20.0;

#[derive(Properties, PartialEq)]
pub struct ScrollAreaProps {
    #[prop_or_default]
    pub children: Children,

    /// Which axes get custom scrollbars. Default: Vertical.
    #[prop_or_default]
    pub direction: ScrollDirection,

    /// CSS classes applied to the inner viewport div.
    #[prop_or_default]
    pub class: Classes,

    /// HTML id applied to the inner viewport div.
    #[prop_or_default]
    pub id: Option<AttrValue>,

    /// CSS classes applied to the outer wrapper div.
    #[prop_or_default]
    pub outer_class: Classes,

    /// Inline style applied to the outer wrapper div (for flex-item props etc.).
    #[prop_or_default]
    pub outer_style: Option<AttrValue>,

    /// Forward a NodeRef to the viewport element for external scroll control.
    #[prop_or_default]
    pub viewport_ref: Option<NodeRef>,
}

// ---------------------------------------------------------------------------
// Thumb geometry helpers
// ---------------------------------------------------------------------------

fn compute_y(vp: &Element) -> Option<(f64, f64, bool)> {
    let ch = vp.client_height() as f64;
    let sh = vp.scroll_height() as f64;
    if sh <= ch {
        return Some((0.0, 0.0, false));
    }
    let ratio = ch / sh;
    let thumb_len = (ratio * ch).max(MIN_THUMB);
    let scroll_range = sh - ch;
    let track_range = ch - thumb_len;
    let pos = vp.scroll_top() as f64;
    let top = if scroll_range > 0.0 {
        (pos / scroll_range) * track_range
    } else {
        0.0
    };
    Some((top, thumb_len, true))
}

fn compute_x(vp: &Element) -> Option<(f64, f64, bool)> {
    let cw = vp.client_width() as f64;
    let sw = vp.scroll_width() as f64;
    if sw <= cw {
        return Some((0.0, 0.0, false));
    }
    let ratio = cw / sw;
    let thumb_len = (ratio * cw).max(MIN_THUMB);
    let scroll_range = sw - cw;
    let track_range = cw - thumb_len;
    let pos = vp.scroll_left() as f64;
    let left = if scroll_range > 0.0 {
        (pos / scroll_range) * track_range
    } else {
        0.0
    };
    Some((left, thumb_len, true))
}

fn apply_thumb_y(thumb_ref: &NodeRef, top: f64, height: f64) {
    if let Some(el) = thumb_ref.cast::<HtmlElement>() {
        let style = el.style();
        let _ = style.set_property("top", &format!("{top}px"));
        let _ = style.set_property("height", &format!("{height}px"));
    }
}

fn apply_thumb_x(thumb_ref: &NodeRef, left: f64, width: f64) {
    if let Some(el) = thumb_ref.cast::<HtmlElement>() {
        let style = el.style();
        let _ = style.set_property("left", &format!("{left}px"));
        let _ = style.set_property("width", &format!("{width}px"));
    }
}

/// Pairs a reactive state handle with an always-current `use_mut_ref` mirror.
///
/// `UseStateHandle` clones read the value at clone-time and are stale in
/// long-lived closures. The `Rc<RefCell<bool>>` mirror is updated alongside
/// the state and is safe to read from any closure.
struct OverflowState {
    state: UseStateHandle<bool>,
    cur: Rc<RefCell<bool>>,
}

impl OverflowState {
    /// Update if changed; triggers a Yew re-render only when the value flips.
    fn set_if_changed(&self, needed: bool) {
        if *self.cur.borrow() != needed {
            *self.cur.borrow_mut() = needed;
            self.state.set(needed);
        }
    }
}

/// Recalculate thumb geometry and update state/DOM.
fn refresh_all(
    vp: &Element,
    direction: ScrollDirection,
    thumb_y_ref: &NodeRef,
    thumb_x_ref: &NodeRef,
    oy: &OverflowState,
    ox: &OverflowState,
) {
    if matches!(direction, ScrollDirection::Vertical | ScrollDirection::Both)
        && let Some((top, h, needed)) = compute_y(vp)
    {
        oy.set_if_changed(needed);
        if needed {
            apply_thumb_y(thumb_y_ref, top, h);
        }
    }
    if matches!(
        direction,
        ScrollDirection::Horizontal | ScrollDirection::Both
    ) && let Some((left, w, needed)) = compute_x(vp)
    {
        ox.set_if_changed(needed);
        if needed {
            apply_thumb_x(thumb_x_ref, left, w);
        }
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[function_component(ScrollArea)]
pub fn scroll_area(props: &ScrollAreaProps) -> Html {
    let direction = props.direction;

    // -- refs ---------------------------------------------------------------
    let own_viewport_ref = use_node_ref();
    let viewport_ref = props
        .viewport_ref
        .clone()
        .unwrap_or_else(|| own_viewport_ref.clone());
    let thumb_y_ref = use_node_ref();
    let thumb_x_ref = use_node_ref();
    let track_y_ref = use_node_ref();
    let track_x_ref = use_node_ref();

    // -- state --------------------------------------------------------------
    let hovered = use_state(|| false);
    let dragging = use_state(|| false);
    let needs_y = use_state(|| false);
    let needs_x = use_state(|| false);
    let needs_y_cur: Rc<RefCell<bool>> = use_mut_ref(|| false);
    let needs_x_cur: Rc<RefCell<bool>> = use_mut_ref(|| false);
    let drag_data: Rc<RefCell<Option<DragStart>>> = use_mut_ref(|| None);

    // -- scroll + resize + mutation observer effect -------------------------
    {
        let viewport_ref = viewport_ref.clone();
        let thumb_y_ref = thumb_y_ref.clone();
        let thumb_x_ref = thumb_x_ref.clone();
        let oy = OverflowState {
            state: needs_y.clone(),
            cur: needs_y_cur.clone(),
        };
        let ox = OverflowState {
            state: needs_x.clone(),
            cur: needs_x_cur.clone(),
        };

        use_effect_with((), move |_| {
            let Some(vp) = viewport_ref.cast::<Element>() else {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            };

            // Initial geometry
            refresh_all(&vp, direction, &thumb_y_ref, &thumb_x_ref, &oy, &ox);

            // Scroll listener
            let ty = thumb_y_ref.clone();
            let tx = thumb_x_ref.clone();
            let oy1 = OverflowState {
                state: oy.state.clone(),
                cur: oy.cur.clone(),
            };
            let ox1 = OverflowState {
                state: ox.state.clone(),
                cur: ox.cur.clone(),
            };
            let vp_for_scroll = vp.clone();
            let on_scroll = Closure::<dyn FnMut()>::wrap(Box::new(move || {
                refresh_all(&vp_for_scroll, direction, &ty, &tx, &oy1, &ox1);
            }));
            let _ =
                vp.add_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref());

            // ResizeObserver on viewport (catches container resize)
            let ty2 = thumb_y_ref.clone();
            let tx2 = thumb_x_ref.clone();
            let oy2 = OverflowState {
                state: oy.state.clone(),
                cur: oy.cur.clone(),
            };
            let ox2 = OverflowState {
                state: ox.state.clone(),
                cur: ox.cur.clone(),
            };
            let vp_for_ro = vp.clone();
            let ro_cb = Closure::<dyn FnMut(js_sys::Array)>::wrap(Box::new(
                move |_entries: js_sys::Array| {
                    refresh_all(&vp_for_ro, direction, &ty2, &tx2, &oy2, &ox2);
                },
            ));
            let resize_obs = web_sys::ResizeObserver::new(ro_cb.as_ref().unchecked_ref()).ok();
            if let Some(ref obs) = resize_obs {
                obs.observe(&vp);
            }

            // MutationObserver on viewport (catches child add/remove)
            let ty3 = thumb_y_ref.clone();
            let tx3 = thumb_x_ref.clone();
            let oy3 = OverflowState {
                state: oy.state.clone(),
                cur: oy.cur.clone(),
            };
            let ox3 = OverflowState {
                state: ox.state.clone(),
                cur: ox.cur.clone(),
            };
            let vp_for_mo = vp.clone();
            let mo_cb = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::wrap(Box::new(
                move |_entries: js_sys::Array, _observer: MutationObserver| {
                    refresh_all(&vp_for_mo, direction, &ty3, &tx3, &oy3, &ox3);
                },
            ));
            let mutation_obs = MutationObserver::new(mo_cb.as_ref().unchecked_ref()).ok();
            if let Some(ref obs) = mutation_obs {
                let init = MutationObserverInit::new();
                init.set_child_list(true);
                init.set_subtree(true);
                let _ = obs.observe_with_options(&vp, &init);
            }

            // Input listener for contenteditable/form fields
            let ty4 = thumb_y_ref;
            let tx4 = thumb_x_ref;
            let oy4 = oy;
            let ox4 = ox;
            let vp_for_input = vp.clone();
            let on_input = Closure::<dyn FnMut()>::wrap(Box::new(move || {
                refresh_all(&vp_for_input, direction, &ty4, &tx4, &oy4, &ox4);
            }));
            let _ = vp.add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref());

            let vp_cleanup = vp;
            Box::new(move || {
                let _ = vp_cleanup.remove_event_listener_with_callback(
                    "scroll",
                    on_scroll.as_ref().unchecked_ref(),
                );
                let _ = vp_cleanup.remove_event_listener_with_callback(
                    "input",
                    on_input.as_ref().unchecked_ref(),
                );
                if let Some(obs) = resize_obs {
                    obs.disconnect();
                }
                if let Some(obs) = mutation_obs {
                    obs.disconnect();
                }
                drop(on_scroll);
                drop(ro_cb);
                drop(mo_cb);
                drop(on_input);
            }) as Box<dyn FnOnce()>
        });
    }

    // -- post-render geometry effect ----------------------------------------
    {
        let vr = viewport_ref.clone();
        let ty = thumb_y_ref.clone();
        let tx = thumb_x_ref.clone();
        let ny = *needs_y;
        let nx = *needs_x;
        use_effect_with((ny, nx), move |&(ny, nx)| {
            if let Some(vp) = vr.cast::<Element>() {
                if ny && let Some((top, h, _)) = compute_y(&vp) {
                    apply_thumb_y(&ty, top, h);
                }
                if nx && let Some((left, w, _)) = compute_x(&vp) {
                    apply_thumb_x(&tx, left, w);
                }
            }
        });
    }

    // -- drag effect (window-level pointer listeners) -----------------------
    {
        let viewport_ref = viewport_ref.clone();
        let thumb_y_ref = thumb_y_ref.clone();
        let thumb_x_ref = thumb_x_ref.clone();
        let drag_data = drag_data.clone();
        let dragging = dragging.clone();
        let hovered = hovered.clone();
        let oy_drag = OverflowState {
            state: needs_y.clone(),
            cur: needs_y_cur.clone(),
        };
        let ox_drag = OverflowState {
            state: needs_x.clone(),
            cur: needs_x_cur.clone(),
        };

        use_effect_with(*dragging, move |is_dragging| {
            if !is_dragging {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            }

            let Some(win) = web_sys::window() else {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            };

            let dd = drag_data.clone();
            let vr = viewport_ref.clone();
            let ty = thumb_y_ref.clone();
            let tx = thumb_x_ref.clone();
            let oy = OverflowState {
                state: oy_drag.state.clone(),
                cur: oy_drag.cur.clone(),
            };
            let ox = OverflowState {
                state: ox_drag.state.clone(),
                cur: ox_drag.cur.clone(),
            };

            let on_move =
                Closure::<dyn FnMut(PointerEvent)>::wrap(Box::new(move |e: PointerEvent| {
                    e.prevent_default();
                    let borrow = dd.borrow();
                    let Some(ds) = borrow.as_ref() else { return };
                    let Some(vp) = vr.cast::<Element>() else {
                        return;
                    };

                    let current = match ds.axis {
                        DragAxis::Y => e.client_y() as f64,
                        DragAxis::X => e.client_x() as f64,
                    };
                    let delta_px = current - ds.pointer_origin;
                    let usable_track = ds.track_len - ds.thumb_len;
                    if usable_track <= 0.0 {
                        return;
                    }
                    let scroll_delta = (delta_px / usable_track) * ds.scroll_range;
                    let new_scroll = (ds.scroll_origin + scroll_delta).clamp(0.0, ds.scroll_range);

                    match ds.axis {
                        DragAxis::Y => vp.set_scroll_top(new_scroll as i32),
                        DragAxis::X => vp.set_scroll_left(new_scroll as i32),
                    }

                    refresh_all(&vp, direction, &ty, &tx, &oy, &ox);
                }));

            let dragging_up = dragging.clone();
            let hovered_up = hovered.clone();
            let on_up =
                Closure::<dyn FnMut(PointerEvent)>::wrap(Box::new(move |_e: PointerEvent| {
                    dragging_up.set(false);
                    let _ = &hovered_up; // keep alive
                }));

            let _ = win
                .add_event_listener_with_callback("pointermove", on_move.as_ref().unchecked_ref());
            let _ =
                win.add_event_listener_with_callback("pointerup", on_up.as_ref().unchecked_ref());

            let win_cleanup = win;
            Box::new(move || {
                let _ = win_cleanup.remove_event_listener_with_callback(
                    "pointermove",
                    on_move.as_ref().unchecked_ref(),
                );
                let _ = win_cleanup.remove_event_listener_with_callback(
                    "pointerup",
                    on_up.as_ref().unchecked_ref(),
                );
                drop(on_move);
                drop(on_up);
            }) as Box<dyn FnOnce()>
        });
    }

    // -- callbacks ----------------------------------------------------------

    let on_enter = {
        let hovered = hovered.clone();
        let viewport_ref = viewport_ref.clone();
        let thumb_y_ref = thumb_y_ref.clone();
        let thumb_x_ref = thumb_x_ref.clone();
        let oy = OverflowState {
            state: needs_y.clone(),
            cur: needs_y_cur.clone(),
        };
        let ox = OverflowState {
            state: needs_x.clone(),
            cur: needs_x_cur.clone(),
        };
        Callback::from(move |_: PointerEvent| {
            hovered.set(true);
            if let Some(vp) = viewport_ref.cast::<Element>() {
                refresh_all(&vp, direction, &thumb_y_ref, &thumb_x_ref, &oy, &ox);
            }
        })
    };

    let on_leave = {
        let hovered = hovered.clone();
        let dragging = dragging.clone();
        Callback::from(move |_: PointerEvent| {
            if !*dragging {
                hovered.set(false);
            }
        })
    };

    let make_thumb_down = |axis: DragAxis| {
        let drag_data = drag_data.clone();
        let dragging = dragging.clone();
        let viewport_ref = viewport_ref.clone();
        let track_y_ref = track_y_ref.clone();
        let track_x_ref = track_x_ref.clone();
        Callback::from(move |e: PointerEvent| {
            e.prevent_default();
            e.stop_propagation();

            // Capture pointer on the thumb element
            if let Some(target) = e.target()
                && let Ok(el) = target.dyn_into::<Element>()
            {
                let _ = el.set_pointer_capture(e.pointer_id());
            }

            let Some(vp) = viewport_ref.cast::<Element>() else {
                return;
            };

            let (pointer_origin, scroll_origin, scroll_range, track_len, thumb_len) = match axis {
                DragAxis::Y => {
                    let ch = vp.client_height() as f64;
                    let sh = vp.scroll_height() as f64;
                    let ratio = ch / sh;
                    let tl = track_y_ref
                        .cast::<Element>()
                        .map(|e| e.client_height() as f64)
                        .unwrap_or(ch);
                    let th = (ratio * tl).max(MIN_THUMB);
                    (e.client_y() as f64, vp.scroll_top() as f64, sh - ch, tl, th)
                }
                DragAxis::X => {
                    let cw = vp.client_width() as f64;
                    let sw = vp.scroll_width() as f64;
                    let ratio = cw / sw;
                    let tl = track_x_ref
                        .cast::<Element>()
                        .map(|e| e.client_width() as f64)
                        .unwrap_or(cw);
                    let th = (ratio * tl).max(MIN_THUMB);
                    (
                        e.client_x() as f64,
                        vp.scroll_left() as f64,
                        sw - cw,
                        tl,
                        th,
                    )
                }
            };

            *drag_data.borrow_mut() = Some(DragStart {
                axis,
                pointer_origin,
                scroll_origin,
                scroll_range,
                track_len,
                thumb_len,
            });
            dragging.set(true);
        })
    };

    let on_thumb_y_down = make_thumb_down(DragAxis::Y);
    let on_thumb_x_down = make_thumb_down(DragAxis::X);

    let make_track_click = |axis: DragAxis| {
        let viewport_ref = viewport_ref.clone();
        let thumb_y_ref = thumb_y_ref.clone();
        let thumb_x_ref = thumb_x_ref.clone();
        let oy = OverflowState {
            state: needs_y.clone(),
            cur: needs_y_cur.clone(),
        };
        let ox = OverflowState {
            state: needs_x.clone(),
            cur: needs_x_cur.clone(),
        };
        Callback::from(move |e: MouseEvent| {
            // Only respond to clicks directly on the track, not on the thumb
            if e.target() != e.current_target() {
                return;
            }
            let Some(vp) = viewport_ref.cast::<Element>() else {
                return;
            };
            let Some(target) = e.current_target() else {
                return;
            };
            let Ok(track_el) = target.dyn_into::<Element>() else {
                return;
            };
            let track_rect = track_el.get_bounding_client_rect();

            match axis {
                DragAxis::Y => {
                    let click_y = e.client_y() as f64 - track_rect.top();
                    let ch = vp.client_height() as f64;
                    let sh = vp.scroll_height() as f64;
                    let scroll_range = sh - ch;
                    let ratio = click_y / track_rect.height();
                    let target_scroll = (ratio * scroll_range).clamp(0.0, scroll_range);
                    vp.set_scroll_top(target_scroll as i32);
                }
                DragAxis::X => {
                    let click_x = e.client_x() as f64 - track_rect.left();
                    let cw = vp.client_width() as f64;
                    let sw = vp.scroll_width() as f64;
                    let scroll_range = sw - cw;
                    let ratio = click_x / track_rect.width();
                    let target_scroll = (ratio * scroll_range).clamp(0.0, scroll_range);
                    vp.set_scroll_left(target_scroll as i32);
                }
            }

            refresh_all(&vp, direction, &thumb_y_ref, &thumb_x_ref, &oy, &ox);
        })
    };

    let on_track_y_click = make_track_click(DragAxis::Y);
    let on_track_x_click = make_track_click(DragAxis::X);

    // -- render -------------------------------------------------------------
    let show_y = matches!(direction, ScrollDirection::Vertical | ScrollDirection::Both) && *needs_y;
    let show_x = matches!(
        direction,
        ScrollDirection::Horizontal | ScrollDirection::Both
    ) && *needs_x;

    let outer_class = classes!(
        "scroll-area",
        props.outer_class.clone(),
        (*hovered || *dragging).then_some("scroll-area--hovered"),
        (*dragging).then_some("scroll-area--dragging"),
    );

    let viewport_class = classes!("scroll-area__viewport", props.class.clone());

    let outer_style = props.outer_style.clone().unwrap_or_default();

    html! {
        <div class={outer_class}
             style={outer_style}
             onpointerenter={on_enter}
             onpointerleave={on_leave}>
            <div class={viewport_class}
                 id={props.id.clone()}
                 ref={viewport_ref}>
                { for props.children.iter() }
            </div>
            if show_y {
                <div class="scroll-area__track-y"
                     ref={track_y_ref}
                     onclick={on_track_y_click}>
                    <div class="scroll-area__thumb-y"
                         ref={thumb_y_ref}
                         onpointerdown={on_thumb_y_down} />
                </div>
            }
            if show_x {
                <div class="scroll-area__track-x"
                     ref={track_x_ref}
                     onclick={on_track_x_click}>
                    <div class="scroll-area__thumb-x"
                         ref={thumb_x_ref}
                         onpointerdown={on_thumb_x_down} />
                </div>
            }
        </div>
    }
}
