use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{window, IntersectionObserver, IntersectionObserverInit};
use yew::prelude::*;

#[function_component(TopMenu)]
pub fn top_menu() -> Html {
    let logo_visible = use_state(|| false);

    {
        let logo_visible = logo_visible.clone();
        use_effect_with((), move |_| {
            let Some(win) = window() else {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            };
            let doc = win.document().unwrap();

            // The element to watch in the main content area
            let Some(target) = doc.get_element_by_id("top-text-container") else {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            };

            // Use the custom scroll viewport as the observer root
            let root = doc.query_selector(".scroll-area__viewport").ok().flatten();

            let cb = Closure::<dyn FnMut(js_sys::Array, IntersectionObserver)>::wrap(Box::new(
                move |entries: js_sys::Array, _obs: IntersectionObserver| {
                    if let Some(entry) = entries.get(0).dyn_ref::<web_sys::IntersectionObserverEntry>() {
                        logo_visible.set(!entry.is_intersecting());
                    }
                },
            ));

            let mut opts = IntersectionObserverInit::new();
            if let Some(ref r) = root {
                opts.root(Some(r));
            }

            let observer: Option<IntersectionObserver> =
                IntersectionObserver::new_with_options(cb.as_ref().unchecked_ref(), &opts).ok();

            if let Some(ref obs) = observer {
                obs.observe(&target);
            }

            Box::new(move || {
                if let Some(obs) = observer {
                    obs.disconnect();
                }
                drop(cb);
            }) as Box<dyn FnOnce()>
        });
    }

    let on_home_click = Callback::from(|_: MouseEvent| {
        if let Some(w) = window() {
            let _ = w.location().reload();
        }
    });

    let logo_class = if *logo_visible {
        "logo-container logo-container--visible"
    } else {
        "logo-container"
    };

    html! {
        <div id="top-menu-container" class="no-select">
            <div class="top-menu-button" onclick={on_home_click}>
                <p>{ "Home" }</p>
            </div>
            <div class="top-menu-button">
                <p>{ "Contact Us" }</p>
            </div>

            <div class={logo_class}>
                <img src="assets/lufkins-logo.svg"/>
                <div class="title-container-reg">
                    <h1 class="top-text-title-reg">{ "Lufkins" }</h1>
                    <p>
                        <span>{ "AI" }</span> { " &" } <span>{ " Robotics" }</span>
                    </p>
                </div>
            </div>
        </div>
    }
}
