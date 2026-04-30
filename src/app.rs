use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;
use yew::prelude::*;
use crate::components::{
    TopMenu, Panel, NodeWeb, NeuralWeb, TopText, TopTextAlt
};

const PANEL_COUNT: usize = 2;
const CYCLE_MS: i32 = 20_000;

#[function_component(App)]
pub fn app() -> Html {
    let active = use_state(|| 0usize);

    {
        let active = active.clone();
        use_effect_with((), move |_| {
            let cb = Closure::<dyn FnMut()>::wrap(Box::new(move || {
                active.set((*active + 1) % PANEL_COUNT);
            }));

            let id = window()
                .unwrap()
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    CYCLE_MS,
                )
                .unwrap();

            Box::new(move || {
                let _ = window().unwrap().clear_interval_with_handle(id);
                drop(cb);
            }) as Box<dyn FnOnce()>
        });
    }

    let translate = format!("transform: translateY(-{}vh);", *active * 100);

    html! {
        <div id="container">
            <TopMenu />
            <TopText/>
            <TopTextAlt/>
            // <div class="panel-carousel" style={translate}>
            //     <Panel
            //         title="Title"
            //         text="Some text describing something related to the graphic"
            //         graphic={html! { <NodeWeb /> }}
            //     />
            //     <Panel
            //         title="Title"
            //         text="Some text describing something related to the graphic"
            //         graphic={html! { <NeuralWeb layers={vec![9, 9, 9]} /> }}
            //         panel_type="alt"
            //     />
            // </div>
        </div>
    }
}
