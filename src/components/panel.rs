use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PanelProps {
    pub title: AttrValue,
    pub text: AttrValue,
    pub graphic: Html,
    #[prop_or_default]
    pub panel_type: Option<AttrValue>,
}

#[function_component(Panel)]
pub fn panel(props: &PanelProps) -> Html {
    let type_class = props.panel_type.as_ref().map(|t| t.to_string()).unwrap_or_default();

    html! {
        <div class={classes!("panel-container", "no-select", &type_class)}>
            <div class={classes!("panel-text-container", type_class)}>
                <h1>{ &props.title }</h1>
                <p>{ &props.text }</p>
            </div>
            { props.graphic.clone() }
        </div>
    }
}
