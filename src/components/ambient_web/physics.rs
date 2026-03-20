use std::cell::RefCell;
use std::rc::Rc;

use gloo_render::{request_animation_frame, AnimationFrame};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::NodeRef;

const PADDING: f64 = 20.0;
const MIN_SPEED: f64 = 0.3;
const MAX_SPEED: f64 = 0.8;
const FADE_SPEED_MIN: f64 = 0.003;
const FADE_SPEED_MAX: f64 = 0.008;

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    opacity: f64,
    fade_phase: f64,
    fade_speed: f64,
    hovered: bool,
}

pub struct AnimState {
    particles: Vec<Particle>,
    width: f64,
    height: f64,
    pub cancelled: bool,
    _handle: Option<AnimationFrame>,
}

impl AnimState {
    pub fn set_hovered(&mut self, index: usize, hovered: bool) {
        if let Some(p) = self.particles.get_mut(index) {
            p.hovered = hovered;
        }
    }
}

type TickFn = Rc<RefCell<Option<Box<dyn FnMut()>>>>;

/// Deterministic hash for per-particle variety.
fn hash(index: usize, seed: usize) -> f64 {
    ((index * 13 + seed * 7 + 3) % 101) as f64 / 100.0
}

fn init_particles(node_count: usize, width: f64, height: f64) -> Vec<Particle> {
    (0..node_count)
        .map(|i| {
            let h1 = hash(i, 1);
            let h2 = hash(i, 2);
            let h3 = hash(i, 3);
            let h4 = hash(i, 4);
            let h5 = hash(i, 5);
            let h6 = hash(i, 6);

            let x = PADDING + h1 * (width - 2.0 * PADDING);
            let y = PADDING + h2 * (height - 2.0 * PADDING);

            let speed = MIN_SPEED + h3 * (MAX_SPEED - MIN_SPEED);
            let angle = h4 * std::f64::consts::TAU;
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;

            let fade_phase = h5 * std::f64::consts::TAU;
            let fade_speed = FADE_SPEED_MIN + h6 * (FADE_SPEED_MAX - FADE_SPEED_MIN);

            Particle {
                x,
                y,
                vx,
                vy,
                opacity: 0.5 + 0.5 * fade_phase.sin(),
                fade_phase,
                fade_speed,
                hovered: false,
            }
        })
        .collect()
}

fn step(particles: &mut [Particle], width: f64, height: f64) {
    for p in particles.iter_mut() {
        if p.hovered {
            // Lerp opacity toward 1.0 when hovered
            p.opacity += (1.0 - p.opacity) * 0.1;
            continue;
        }

        // Move
        p.x += p.vx;
        p.y += p.vy;

        // Bounce off edges
        if p.x < PADDING {
            p.x = PADDING;
            p.vx = p.vx.abs();
        } else if p.x > width - PADDING {
            p.x = width - PADDING;
            p.vx = -(p.vx.abs());
        }

        if p.y < PADDING {
            p.y = PADDING;
            p.vy = p.vy.abs();
        } else if p.y > height - PADDING {
            p.y = height - PADDING;
            p.vy = -(p.vy.abs());
        }

        // Fade cycle
        p.fade_phase += p.fade_speed;
        p.opacity = 0.5 + 0.5 * p.fade_phase.sin();
    }
}

fn apply_node_positions(container: &HtmlElement, particles: &[Particle]) {
    let Ok(nodes) = container.query_selector_all(".ambient-web-node") else {
        return;
    };
    let any_hovered = particles.iter().any(|p| p.hovered);
    // Elevate the entire container when any node is hovered so it breaks
    // out of its stacking context and sits above title/menu elements.
    if any_hovered {
        let _ = container.set_attribute("class", "ambient-web no-select has-hover");
    } else {
        let _ = container.set_attribute("class", "ambient-web no-select");
    }

    for i in 0..nodes.length().min(particles.len() as u32) {
        if let Some(el) = nodes.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
            let p = &particles[i as usize];
            // Position so the circle center (top of the flex column) lands at (x, y)
            let _ = el.style().set_property(
                "transform",
                &format!("translate(calc({}px - 50%), calc({}px - 12.5px))", p.x, p.y),
            );
            let _ = el.style().set_property("opacity", &format!("{:.3}", p.opacity));
            if p.hovered {
                let _ = el.set_attribute("class", "ambient-web-node hovered");
            } else {
                let _ = el.set_attribute("class", "ambient-web-node");
            }
        }
    }
}

fn apply_line_positions(
    container: &HtmlElement,
    particles: &[Particle],
    edges: &[(usize, usize)],
) {
    let Ok(lines) = container.query_selector_all(".ambient-web-line") else {
        return;
    };
    for i in 0..lines.length().min(edges.len() as u32) {
        if let Some(el) = lines.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
            let (from, to) = edges[i as usize];
            if from >= particles.len() || to >= particles.len() {
                continue;
            }
            let (x1, y1) = (particles[from].x, particles[from].y);
            let (x2, y2) = (particles[to].x, particles[to].y);
            let dx = x2 - x1;
            let dy = y2 - y1;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx) * 180.0 / std::f64::consts::PI;
            let line_opacity = particles[from].opacity.min(particles[to].opacity);

            let _ = el.style().set_property("width", &format!("{dist}px"));
            let _ = el.style().set_property(
                "transform",
                &format!("translate({x1}px, {y1}px) rotate({angle}deg)"),
            );
            let _ = el.style().set_property("opacity", &format!("{:.3}", line_opacity));
        }
    }
}

pub fn start_animation(
    container_ref: NodeRef,
    node_count: usize,
    width: f64,
    height: f64,
    edges: Vec<(usize, usize)>,
) -> Rc<RefCell<AnimState>> {
    let particles = init_particles(node_count, width, height);

    let state = Rc::new(RefCell::new(AnimState {
        particles,
        width,
        height,
        cancelled: false,
        _handle: None,
    }));

    let tick: TickFn = Rc::new(RefCell::new(None));
    let tick_for_closure = tick.clone();
    let state_for_tick = state.clone();

    *tick.borrow_mut() = Some(Box::new(move || {
        if state_for_tick.borrow().cancelled {
            return;
        }

        {
            let mut st = state_for_tick.borrow_mut();
            let (w, h) = (st.width, st.height);
            step(&mut st.particles, w, h);
        }

        if let Some(container) = container_ref.cast::<HtmlElement>() {
            let st = state_for_tick.borrow();
            apply_node_positions(&container, &st.particles);
            apply_line_positions(&container, &st.particles, &edges);
        }

        let tick_next = tick_for_closure.clone();
        let handle = request_animation_frame(move |_| {
            if let Some(cb) = tick_next.borrow_mut().as_mut() {
                cb();
            }
        });
        state_for_tick.borrow_mut()._handle = Some(handle);
    }));

    {
        let tick_first = tick.clone();
        let handle = request_animation_frame(move |_| {
            if let Some(cb) = tick_first.borrow_mut().as_mut() {
                cb();
            }
        });
        state.borrow_mut()._handle = Some(handle);
    }

    state
}
