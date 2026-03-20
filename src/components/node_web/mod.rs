mod data;
pub mod physics;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::PointerEvent;
use yew::prelude::*;

#[function_component(NodeWeb)]
pub fn node_web() -> Html {
    let container_ref = use_node_ref();
    let sim_state = use_mut_ref(|| None::<Rc<RefCell<physics::SimState>>>);

    let graph = data::load();
    let node_count = graph.nodes.len();
    let edges = graph.edges.clone();

    // Start the force simulation on mount
    {
        let container_ref = container_ref.clone();
        let sim_state = sim_state.clone();
        let edges = edges.clone();
        use_effect(move || {
            let (width, height) = if let Some(el) = container_ref.cast::<web_sys::Element>() {
                let rect = el.get_bounding_client_rect();
                (rect.width(), rect.height())
            } else {
                (800.0, 500.0)
            };

            let state = physics::start_simulation(
                container_ref,
                node_count,
                width,
                height,
                width / 100.0 * 65.0,
                height / 2.0,
                edges,
            );
            *sim_state.borrow_mut() = Some(state);

            move || {
                if let Some(st) = sim_state.borrow().as_ref() {
                    st.borrow_mut().cancelled = true;
                }
            }
        });
    }

    // ResizeObserver to rescale on container resize
    {
        let container_ref = container_ref.clone();
        let sim_state = sim_state.clone();
        let edges = edges.clone();
        use_effect_with((), move |_| {
            let Some(el) = container_ref.cast::<web_sys::Element>() else {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            };

            let container_ref2 = container_ref.clone();
            let ro_cb = Closure::<dyn FnMut(js_sys::Array)>::wrap(Box::new(
                move |_entries: js_sys::Array| {
                    let Some(el) = container_ref2.cast::<web_sys::Element>() else { return };
                    let rect = el.get_bounding_client_rect();
                    let w = rect.width();
                    let h = rect.height();
                    if let Some(state) = sim_state.borrow().as_ref() {
                        let st = state.borrow();
                        // Only resize if dimensions actually changed
                        if (st.width - w).abs() > 1.0 || (st.height - h).abs() > 1.0 {
                            drop(st);
                            physics::resize(state, w, h, &container_ref2, &edges);
                        }
                    }
                },
            ));
            let observer = web_sys::ResizeObserver::new(ro_cb.as_ref().unchecked_ref()).ok();
            if let Some(ref obs) = observer {
                obs.observe(&el);
            }

            Box::new(move || {
                if let Some(obs) = observer {
                    obs.disconnect();
                }
                drop(ro_cb);
            }) as Box<dyn FnOnce()>
        });
    }

    // Drag handlers
    let drag_index = use_mut_ref(|| None::<usize>);

    let onpointerdown = {
        let sim_state = sim_state.clone();
        let drag_index = drag_index.clone();
        let container_ref = container_ref.clone();
        let edges = edges.clone();
        Callback::from(move |e: PointerEvent| {
            // Find which node was clicked by walking up to [data-index]
            let target = match e.target() {
                Some(t) => t,
                None => return,
            };
            let el = match target.dyn_ref::<web_sys::Element>() {
                Some(el) => el.closest(".node-web-node").ok().flatten(),
                None => None,
            };
            let node_el = match el {
                Some(el) => el,
                None => return,
            };
            let index: usize = match node_el.get_attribute("data-index") {
                Some(s) => match s.parse() {
                    Ok(i) => i,
                    Err(_) => return,
                },
                None => return,
            };

            // Get position relative to container
            let container = match container_ref.cast::<web_sys::Element>() {
                Some(c) => c,
                None => return,
            };
            let rect = container.get_bounding_client_rect();
            let x = e.client_x() as f64 - rect.left();
            let y = e.client_y() as f64 - rect.top();

            // Capture pointer
            let _ = container.set_pointer_capture(e.pointer_id());

            // Add dragging class directly via DOM (no re-render)
            let _ = container.class_list().add_1("dragging");

            *drag_index.borrow_mut() = Some(index);

            if let Some(state) = sim_state.borrow().as_ref() {
                physics::pin_node(state, index, x, y, &container_ref, &edges);
            }
        })
    };

    let onpointermove = {
        let sim_state = sim_state.clone();
        let drag_index = drag_index.clone();
        let container_ref = container_ref.clone();
        Callback::from(move |e: PointerEvent| {
            let index = match *drag_index.borrow() {
                Some(i) => i,
                None => return,
            };

            let container = match container_ref.cast::<web_sys::Element>() {
                Some(c) => c,
                None => return,
            };
            let rect = container.get_bounding_client_rect();
            let x = e.client_x() as f64 - rect.left();
            let y = e.client_y() as f64 - rect.top();

            if let Some(state) = sim_state.borrow().as_ref() {
                physics::move_node(state, index, x, y);
            }
        })
    };

    let onpointerup = {
        let sim_state = sim_state.clone();
        let drag_index = drag_index.clone();
        let container_ref = container_ref.clone();
        let edges = edges.clone();
        Callback::from(move |e: PointerEvent| {
            let index = match drag_index.borrow_mut().take() {
                Some(i) => i,
                None => return,
            };

            if let Some(container) = container_ref.cast::<web_sys::Element>() {
                let _ = container.release_pointer_capture(e.pointer_id());
                let _ = container.class_list().remove_1("dragging");
            }

            if let Some(state) = sim_state.borrow().as_ref() {
                physics::unpin_node(state, index);
                physics::reheat(state, &container_ref, &edges);
            }
        })
    };

    html! {
        <div class="node-web no-select"
             ref={container_ref.clone()}
             onpointerdown={onpointerdown}
             onpointermove={onpointermove}
             onpointerup={onpointerup}
        >
            { for graph.edges.iter().map(|(from, to)| {
                html! {
                    <div class="node-web-line"
                         data-from={from.to_string()}
                         data-to={to.to_string()}
                         key={format!("line-{from}-{to}")} />
                }
            })}
            { for graph.nodes.iter().enumerate().map(|(i, node)| {
                html! {
                    <div class="node-web-node" data-index={i.to_string()} key={format!("node-{i}")}>
                        <div class="node-web-node-circle" />
                        <div class="node-web-node-label">{ &node.label }</div>
                    </div>
                }
            })}
        </div>
    }
}
