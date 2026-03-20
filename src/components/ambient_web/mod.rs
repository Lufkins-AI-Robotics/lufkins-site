mod data;
mod physics;

use yew::prelude::*;

#[function_component(AmbientWeb)]
pub fn ambient_web() -> Html {
    let container_ref = use_node_ref();
    let anim_state = use_mut_ref(|| None::<std::rc::Rc<std::cell::RefCell<physics::AnimState>>>);

    let graph = data::load();
    let node_count = graph.nodes.len();
    let edges_for_effect = graph.edges.clone();

    {
        let container_ref = container_ref.clone();
        let anim_state = anim_state.clone();
        use_effect(move || {
            let (width, height) = if let Some(el) = container_ref.cast::<web_sys::Element>() {
                let rect = el.get_bounding_client_rect();
                (rect.width(), rect.height())
            } else {
                (800.0, 500.0)
            };

            let state = physics::start_animation(
                container_ref,
                node_count,
                width,
                height,
                edges_for_effect,
            );
            *anim_state.borrow_mut() = Some(state);

            move || {
                if let Some(st) = anim_state.borrow().as_ref() {
                    st.borrow_mut().cancelled = true;
                }
            }
        });
    }

    html! {
        <div class="ambient-web no-select" ref={container_ref.clone()}>
            { for graph.edges.iter().map(|(from, to)| {
                html! {
                    <div class="ambient-web-line"
                         data-from={from.to_string()}
                         data-to={to.to_string()}
                         key={format!("line-{from}-{to}")} />
                }
            })}
            { for graph.nodes.iter().enumerate().map(|(i, node)| {
                let anim_enter = anim_state.clone();
                let anim_leave = anim_state.clone();

                let onmouseenter = Callback::from(move |_: MouseEvent| {
                    if let Some(st) = anim_enter.borrow().as_ref() {
                        st.borrow_mut().set_hovered(i, true);
                    }
                });
                let onmouseleave = Callback::from(move |_: MouseEvent| {
                    if let Some(st) = anim_leave.borrow().as_ref() {
                        st.borrow_mut().set_hovered(i, false);
                    }
                });

                html! {
                    <div class="ambient-web-node"
                         data-index={i.to_string()}
                         key={format!("node-{i}")}>
                        <div class="ambient-web-node-circle"
                             {onmouseenter}
                             {onmouseleave} />
                        <div class="ambient-web-node-label">{ &node.label }</div>
                    </div>
                }
            })}
        </div>
    }
}
