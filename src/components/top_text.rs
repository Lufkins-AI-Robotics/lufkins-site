use yew::prelude::*;
use crate::components::{AmbientWeb};

#[function_component(TopText)]
pub fn top_text() -> Html {
    html! {
        <div id="top-text-container" class="no-select">
            // <svg
            //     class="logo-svg"
            //     viewBox="0 0 1092 1207"
            //     xmlns="http://www.w3.org/2000/svg"
            // >
            //     // Top horizontal line
            //     <path
            //         class="logo-stroke stroke-1"
            //         d="m 190.71645,189.14892 h 723.6775"
            //     />
            //     // Vertical line
            //     <path
            //         class="logo-stroke stroke-2"
            //         d="m 552.68941,190.7424 -1.04452,747.06073"
            //     />
            //     // Left diagonal
            //     <path
            //         class="logo-stroke stroke-3"
            //         d="M 516.93153,187.45285 218.56152,823.76034"
            //     />
            //     // Middle horizontal line
            //     <path
            //         class="logo-stroke stroke-4"
            //         d="M 190.21948,505.96977 H 916.50953"
            //     />
            //     // Right diagonal
            //     <path
            //         class="logo-stroke stroke-5"
            //         d="M 590.43723,502.65541 776.97359,904.98875"
            //     />
            //     // Arc (P curve)
            //     <path
            //         class="logo-stroke stroke-6"
            //         d="m 713.07893,188.79965 c 133.00954,0.36947 192.19152,70.56758 191.75542,162.17149 -0.36948,77.61065 -64.67466,154.46512 -190.27754,154.83459"
            //     />
            //     // Lower horizontal line
            //     <path
            //         class="logo-stroke stroke-7"
            //         d="M 536.06111,913.52291 H 916.86955"
            //     />
            //     // Bottom horizontal line
            //     <path
            //         class="logo-stroke stroke-8"
            //         d="m 191.01995,1004.5478 h 723.6775"
            //     />
            // </svg>
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
