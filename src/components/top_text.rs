use yew::prelude::*;
use crate::components::{AmbientWeb};

#[function_component(TopText)]
pub fn top_text() -> Html {
    html! {
        <div id="top-text-container" class="no-select">
            // Option 1: Centered sphere, symmetric warp, gradient grid
            <img class="spacefabric-svg sf-option-1" src="assets/spacefabric-1.svg" alt="spacefabric" />

            // Option 2: Off-center sphere, dense grid, deep funnel, event horizon ring
            <img class="spacefabric-svg sf-option-2" src="assets/spacefabric-2.svg" alt="spacefabric" />

            // Option 3: Angled perspective, color-shifting lines, gravitational lensing rings
            <img class="spacefabric-svg sf-option-3" src="assets/spacefabric-3.svg" alt="spacefabric" />

            <div class="title-container">
                <h1 class="top-text-title">{ "Lufkins" }</h1>
                <p>
                    <span>{ "AI" }</span> { " &" } <span>{ " Robotics" }</span>
                </p>
            </div>

            <div id="blocker"/>

            // <AmbientWeb />
        </div>
    }
}
