use web_sys::window;
use yew::prelude::*;

#[function_component(TopMenu)]
pub fn top_menu() -> Html {
    let on_home_click = Callback::from(|_: MouseEvent| {
        if let Some(w) = window() {
            let _ = w.location().reload();
        }
    });

    html! {
        <div id="top-menu-container" class="no-select">
            <div class="top-menu-button" onclick={on_home_click}>
                <p>{ "Home" }</p>
            </div>
            <div class="top-menu-button">
                <p>{ "Contact Us" }</p>
            </div>

            <div class="logo-container logo-container--visible">
                // <img src="assets/lufkins-logo.svg"/>
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
