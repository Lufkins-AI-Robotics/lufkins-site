use yew::prelude::*;
use crate::components::{
    TopMenu, TopText, Content, Panel, NodeWeb, NeuralWeb, ScrollArea,
};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <ScrollArea outer_class="app-scroll" outer_style="height: 100vh;">
            <div id="container">
                <TopMenu />
                <TopText />
                <Panel
                    title="Title"
                    text="Some text describing something related to the graphic"
                    graphic={html! { <NodeWeb /> }}
                    // panel_type="default"
                />
                <Panel
                    title="Title"
                    text="Some text describing something related to the graphic"
                    graphic={html! { <NeuralWeb layers={vec![9, 9, 9]} /> }}
                    panel_type="alt"
                />
            </div>
        </ScrollArea>
    }
}
