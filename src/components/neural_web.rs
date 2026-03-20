use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use yew::prelude::*;

fn shuffle(slice: &mut [usize]) {
    let mut rng = rand::rng();
    use rand::RngExt;
    for i in (1..slice.len()).rev() {
        let j = rng.random_range(0..=i);
        slice.swap(i, j);
    }
}

#[derive(Properties, PartialEq)]
pub struct NeuralWebProps {
    /// Number of nodes in each column, e.g. `vec![3, 5, 7, 5, 3]`.
    pub layers: Vec<usize>,
}

struct LayoutData {
    cols: Vec<Vec<(f64, f64)>>,
    edges: Vec<(usize, usize, usize, usize)>, // (col, from_row, col+1, to_row)
}

/// Compute column positions from container dimensions. Edges are not recomputed.
fn compute_positions(layers: &[usize], width: f64, height: f64) -> Vec<Vec<(f64, f64)>> {
    let num_cols = layers.len();
    let padding_x = width * 0.1;
    let padding_y = height * 0.1;
    let usable_w = width - padding_x * 2.0;
    let usable_h = height - padding_y * 2.0;
    let curve_strength = usable_w * 0.12;

    layers
        .iter()
        .enumerate()
        .map(|(col, &rows)| {
            let base_x = if num_cols > 1 {
                padding_x + (col as f64 / (num_cols - 1) as f64) * usable_w
            } else {
                width / 2.0
            };
            let direction = if col == 0 {
                -1.0
            } else if col == num_cols - 1 {
                1.0
            } else {
                0.0
            };

            (0..rows)
                .map(|row| {
                    let t = if rows > 1 {
                        (row as f64) / (rows as f64 - 1.0)
                    } else {
                        0.5
                    };
                    let y = padding_y + ((row as f64 + 0.5) / rows as f64) * usable_h;
                    let curve_offset = 4.0 * t * (1.0 - t) * curve_strength * direction;
                    let x = base_x + curve_offset;
                    (x, y)
                })
                .collect()
        })
        .collect()
}

/// Generate random 1:1 edges between adjacent columns.
fn generate_edges(layers: &[usize]) -> Vec<(usize, usize, usize, usize)> {
    let num_cols = layers.len();
    let mut edges = Vec::new();
    for col in 0..(num_cols.saturating_sub(1)) {
        let left = layers[col];
        let right = layers[col + 1];

        if left <= right {
            let mut targets: Vec<usize> = (0..right).collect();
            shuffle(&mut targets);
            for (from_row, &to_row) in targets.iter().enumerate().take(left) {
                edges.push((col, from_row, col + 1, to_row));
            }
        } else {
            let mut sources: Vec<usize> = (0..left).collect();
            shuffle(&mut sources);
            for (to_row, &from_row) in sources.iter().enumerate().take(right) {
                edges.push((col, from_row, col + 1, to_row));
            }
        }
    }
    edges
}

#[function_component(NeuralWeb)]
pub fn neural_web(props: &NeuralWebProps) -> Html {
    let container_ref = use_node_ref();
    let layout = use_state(|| None::<LayoutData>);

    let layers = props.layers.clone();

    // Generate edges once on mount, compute initial positions
    let edges_ref: Rc<RefCell<Option<Vec<(usize, usize, usize, usize)>>>> = use_mut_ref(|| None);
    {
        let container_ref = container_ref.clone();
        let layout = layout.clone();
        let layers = layers.clone();
        let edges_ref = edges_ref.clone();
        use_effect_with((), move |_| {
            let edges = generate_edges(&layers);
            *edges_ref.borrow_mut() = Some(edges.clone());

            let (width, height) = if let Some(el) = container_ref.cast::<web_sys::Element>() {
                let rect = el.get_bounding_client_rect();
                (rect.width(), rect.height())
            } else {
                (800.0, 500.0)
            };

            let cols = compute_positions(&layers, width, height);
            layout.set(Some(LayoutData { cols, edges }));
            || ()
        });
    }

    // ResizeObserver to recompute positions (but keep same edges)
    {
        let container_ref = container_ref.clone();
        let layout = layout.clone();
        let layers = layers.clone();
        let edges_ref = edges_ref.clone();
        use_effect_with((), move |_| {
            let Some(el) = container_ref.cast::<web_sys::Element>() else {
                return Box::new(|| {}) as Box<dyn FnOnce()>;
            };

            let prev_size: Rc<RefCell<(f64, f64)>> = Rc::new(RefCell::new((0.0, 0.0)));

            let container_ref2 = container_ref.clone();
            let ro_cb = Closure::<dyn FnMut(js_sys::Array)>::wrap(Box::new(
                move |_entries: js_sys::Array| {
                    let Some(el) = container_ref2.cast::<web_sys::Element>() else { return };
                    let rect = el.get_bounding_client_rect();
                    let w = rect.width();
                    let h = rect.height();

                    // Only update if dimensions actually changed
                    let (pw, ph) = *prev_size.borrow();
                    if (pw - w).abs() < 1.0 && (ph - h).abs() < 1.0 {
                        return;
                    }
                    *prev_size.borrow_mut() = (w, h);

                    let cols = compute_positions(&layers, w, h);
                    let edges = edges_ref.borrow().clone().unwrap_or_default();
                    layout.set(Some(LayoutData { cols, edges }));
                },
            ));
            let observer = web_sys::ResizeObserver::new(ro_cb.as_ref().unchecked_ref()).ok();
            if let Some(ref obs) = observer {
                obs.observe(&el);
            }

            Box::new(move || {
                if let Some(obs) = observer {
                    obs.disconnect();
                }
                drop(ro_cb);
            }) as Box<dyn FnOnce()>
        });
    }

    let Some(data) = layout.as_ref() else {
        return html! { <div class="neural-web" ref={container_ref} /> };
    };

    let lines = data.edges.iter().enumerate().map(|(i, &(c1, r1, c2, r2))| {
        let (x1, y1) = data.cols[c1][r1];
        let (x2, y2) = data.cols[c2][r2];
        let dx = x2 - x1;
        let dy = y2 - y1;
        let dist = (dx * dx + dy * dy).sqrt();
        let angle = dy.atan2(dx) * 180.0 / std::f64::consts::PI;

        let delay = ((i * 7 + c1 * 13 + r1 * 31 + r2 * 17) % 80) as f64 / 10.0;
        let duration = 1.5 + ((i * 3 + r1 * 11 + c2 * 7) % 20) as f64 / 10.0;

        html! {
            <div class="neural-web-line"
                 style={format!(
                     "width: {dist}px; transform: translate({x1}px, {y1}px) rotate({angle}deg);"
                 )}
                 key={format!("line-{c1}-{r1}-{c2}-{r2}")}
            >
                <div class="neural-web-pulse"
                     style={format!("animation-delay: {delay}s; animation-duration: {duration}s;")}
                />
            </div>
        }
    });

    let nodes = data.cols.iter().enumerate().flat_map(|(col, col_positions)| {
        col_positions.iter().enumerate().map(move |(row, &(x, y))| {
            html! {
                <div class="neural-web-node"
                     style={format!("transform: translate(calc({x}px - 50%), calc({y}px - 50%));")}
                     key={format!("node-{col}-{row}")}
                >
                    <div class="neural-web-node-circle" />
                </div>
            }
        })
    });

    html! {
        <div class="neural-web no-select" ref={container_ref}>
            { for lines }
            { for nodes }
        </div>
    }
}
