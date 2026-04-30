use yew::prelude::*;
use crate::components::{AmbientWeb};

#[function_component(TopText)]
pub fn top_text() -> Html {
    html! {
        <div id="top-text-container" class="no-select">
            // // Option 1: Centered sphere, symmetric warp, gradient grid
            // <img class="spacefabric-svg sf-option-1" src="assets/spacefabric-1.svg" alt="spacefabric" />

            // // Option 2: Off-center sphere, dense grid, deep funnel, event horizon ring
            // <img class="spacefabric-svg sf-option-2" src="assets/spacefabric-2.svg" alt="spacefabric" />

            // // Option 3: Angled perspective, color-shifting lines, gravitational lensing rings
            // <img class="spacefabric-svg sf-option-3" src="assets/spacefabric-3.svg" alt="spacefabric" />

            <div class="sidebar">
                <div class="sidebar-dot"/>
            </div>

            <div class="title-container">

                <h1>{ "Who we are" }</h1>

                <p>{ "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum." }</p>
            </div>

            // <div id="blocker"/>

            // <AmbientWeb />
        </div>
    }
}
