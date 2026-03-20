use yew::prelude::*;

#[function_component(Content)]
pub fn content() -> Html {
    html! {
        <div id="content-container" class="no-select">
            <h1>{ "Entity Management Solutions" }</h1>
            <p>{"Builds clean. Here's what changed:

physics.rs — significantly simplified:

Removed ambient phase entirely (no Phase enum, no ambient_step, no velocities)
Distances are now deterministic via node_distance() — a hash-based function that maps each node index to a consistent distance between 120–380px
The rAF loop runs only during the eased pop-out, then stops once all nodes settle
Renamed PhysicsState → AnimState since it's no longer really physics
Removed js_sys::Math::random() — no runtime randomness for positions
Sizes, colours, and labels are still fully customizable from mod.rs and nodes.json. Angles are evenly spaced (i * TAU / n), distances are deterministic per index, and the whole thing is resolved at build time.

Could you create a new css variable in the _variables.scss file and assign is as the background color for these two elements:
top-menu-container
top-text-container

I want to achieve this visual:
-there is no distinction between the top menu and top text
-as the user scrolls down, I want the top menu have a vertical gradient so that text and things that are in the content-container appear to fade to white (or whatever the colour is)Done. Here's what I did:

_variables.scss:35 — Added $top-bg: $off-white; so you can change the shared colour in one place.
top_text.scss:13 — Replaced background-color: orange with $top-bg.
top_menu.scss:13 — Added a vertical gradient: solid $top-bg at the top (seamless with top-text), transparent in the middle (buttons live here), then solid $top-bg at the bottom edge. Content scrolling up underneath will fade into the background colour before disappearing.
The gradient stops are at 0%/40%/60%/100% — you can adjust those to control how much of the fade-in/fade-out region you want. If you want a sharper fade at the bottom, push the 60% stop closer to 100%.

Perfect, now I want it so that the content-container takes up the remainder of the vertical space on the page south of the top-text-container.Done. Three changes:

container.scss — Added display: flex; flex-direction: column so children stack vertically.
top_text.scss — Changed from position: absolute; top: 0 to position: relative; flex-shrink: 0. It keeps its height: 60%; max-height: 600px but now participates in flow, and flex-shrink: 0 prevents it from being compressed.
content.scss — Replaced height: 100% with flex: 1; min-height: 0 so it fills whatever vertical space remains below the top-text."}</p>
        </div>
    }
}
