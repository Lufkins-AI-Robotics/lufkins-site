use std::cell::RefCell;
use std::rc::Rc;

use gloo_render::{request_animation_frame, AnimationFrame};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::NodeRef;

// --- Force simulation parameters (at reference size REF_SIZE) ---
const REF_SIZE: f64 = 500.0;
const BASE_REPULSION: f64 = 5_000.0;
const SPRING_K: f64 = 0.05;
const BASE_REST_LENGTH: f64 = 50.0;
const DAMPING: f64 = 0.85;
const CENTER_STRENGTH: f64 = 0.003;
const INITIAL_TEMP: f64 = 100.0;
const COOLING: f64 = 0.995;
const MIN_TEMP: f64 = 0.3;
const BASE_PADDING: f64 = 40.0;

// Y offset from node div center to circle center.
const CIRCLE_Y_OFFSET: f64 = -12.0;

// Drag reheat temperature
const REHEAT_TEMP: f64 = 40.0;

pub struct Particle {
    pub x: f64,
    pub y: f64,
    vx: f64,
    vy: f64,
    pinned: bool,
}

pub struct SimState {
    pub particles: Vec<Particle>,
    pub edges: Vec<(usize, usize)>,
    pub width: f64,
    pub height: f64,
    pub center_x: f64,
    pub center_y: f64,
    pub scale: f64,
    temperature: f64,
    pub cancelled: bool,
    settled: bool,
    pub dragging: Option<usize>,
    pub drag_offset_x: f64,
    pub drag_offset_y: f64,
    _handle: Option<AnimationFrame>,
}

type TickFn = Rc<RefCell<Option<Box<dyn FnMut()>>>>;

/// Deterministic initial placement in a circle around origin.
fn initial_position(index: usize, count: usize, center_x: f64, center_y: f64, radius: f64) -> (f64, f64) {
    let angle = (index as f64) * std::f64::consts::TAU / (count as f64);
    let hash = ((index * 7 + 3) % 10) as f64 / 9.0;
    let r = radius * (0.5 + 0.5 * hash);
    (center_x + angle.cos() * r, center_y + angle.sin() * r)
}

fn force_step(state: &mut SimState) {
    let n = state.particles.len();
    if n == 0 {
        return;
    }

    let s = state.scale;
    let repulsion = BASE_REPULSION * s * s;
    let rest_length = BASE_REST_LENGTH * s;
    let padding = BASE_PADDING * s;

    let cx = state.center_x;
    let cy = state.center_y;

    // Accumulate forces
    let mut fx = vec![0.0f64; n];
    let mut fy = vec![0.0f64; n];

    // Repulsion (O(n²) — fine for ~10-20 nodes)
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = state.particles[j].x - state.particles[i].x;
            let dy = state.particles[j].y - state.particles[i].y;
            let dist_sq = dx * dx + dy * dy;
            let dist = dist_sq.sqrt().max(1.0);
            let force = repulsion / dist_sq.max(1.0);
            let fdx = (dx / dist) * force;
            let fdy = (dy / dist) * force;
            fx[i] -= fdx;
            fy[i] -= fdy;
            fx[j] += fdx;
            fy[j] += fdy;
        }
    }

    // Spring attraction along edges
    for &(from, to) in &state.edges {
        if from >= n || to >= n {
            continue;
        }
        let dx = state.particles[to].x - state.particles[from].x;
        let dy = state.particles[to].y - state.particles[from].y;
        let dist = (dx * dx + dy * dy).sqrt().max(1.0);
        let displacement = dist - rest_length;
        let force = SPRING_K * displacement;
        let fdx = (dx / dist) * force;
        let fdy = (dy / dist) * force;
        fx[from] += fdx;
        fy[from] += fdy;
        fx[to] -= fdx;
        fy[to] -= fdy;
    }

    // Center gravity
    for i in 0..n {
        let dx = cx - state.particles[i].x;
        let dy = cy - state.particles[i].y;
        fx[i] += dx * CENTER_STRENGTH;
        fy[i] += dy * CENTER_STRENGTH;
    }

    // Apply forces, damping, temperature capping
    let mut max_v = 0.0f64;
    for i in 0..n {
        let p = &mut state.particles[i];
        if p.pinned {
            p.vx = 0.0;
            p.vy = 0.0;
            continue;
        }

        p.vx = (p.vx + fx[i]) * DAMPING;
        p.vy = (p.vy + fy[i]) * DAMPING;

        // Cap displacement by temperature
        let speed = (p.vx * p.vx + p.vy * p.vy).sqrt();
        if speed > state.temperature {
            let cap = state.temperature / speed;
            p.vx *= cap;
            p.vy *= cap;
        }

        p.x += p.vx;
        p.y += p.vy;

        // Soft boundary containment
        p.x = p.x.clamp(padding, state.width - padding);
        p.y = p.y.clamp(padding, state.height - padding);

        max_v = max_v.max(speed);
    }

    // Cool down
    state.temperature *= COOLING;
    if state.temperature < MIN_TEMP && max_v < MIN_TEMP && state.dragging.is_none() {
        state.settled = true;
    }
}

fn apply_node_positions(container: &HtmlElement, particles: &[Particle]) {
    let Ok(nodes) = container.query_selector_all(".node-web-node") else {
        return;
    };
    for i in 0..nodes.length().min(particles.len() as u32) {
        if let Some(el) = nodes.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
            let p = &particles[i as usize];
            let _ = el.style().set_property(
                "transform",
                &format!("translate(calc({}px - 50%), calc({}px - 50%))", p.x, p.y),
            );
        }
    }
}

fn apply_line_positions(
    container: &HtmlElement,
    particles: &[Particle],
    edges: &[(usize, usize)],
) {
    let Ok(lines) = container.query_selector_all(".node-web-line") else {
        return;
    };
    for i in 0..lines.length().min(edges.len() as u32) {
        if let Some(el) = lines.item(i).and_then(|n| n.dyn_into::<HtmlElement>().ok()) {
            let (from, to) = edges[i as usize];
            if from >= particles.len() || to >= particles.len() {
                continue;
            }
            let (x1, y1) = (particles[from].x, particles[from].y + CIRCLE_Y_OFFSET);
            let (x2, y2) = (particles[to].x, particles[to].y + CIRCLE_Y_OFFSET);
            let dx = x2 - x1;
            let dy = y2 - y1;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx) * 180.0 / std::f64::consts::PI;

            let _ = el.style().set_property("width", &format!("{dist}px"));
            let _ = el.style().set_property(
                "transform",
                &format!("translate({x1}px, {y1}px) rotate({angle}deg)"),
            );
        }
    }
}

/// Rescale all particle positions to a new container size and reheat.
pub fn resize(state: &Rc<RefCell<SimState>>, new_width: f64, new_height: f64, container_ref: &NodeRef, edges: &[(usize, usize)]) {
    {
        let mut st = state.borrow_mut();
        let scale_x = if st.width > 0.0 { new_width / st.width } else { 1.0 };
        let scale_y = if st.height > 0.0 { new_height / st.height } else { 1.0 };

        for p in &mut st.particles {
            p.x *= scale_x;
            p.y *= scale_y;
        }

        st.center_x *= scale_x;
        st.center_y *= scale_y;
        st.width = new_width;
        st.height = new_height;
        st.scale = new_width.min(new_height) / REF_SIZE;
        st.temperature = REHEAT_TEMP;
        st.settled = false;
    }
    // Update CSS scale variable
    if let Some(el) = container_ref.cast::<HtmlElement>() {
        let s = state.borrow().scale;
        let _ = el.style().set_property("--node-scale", &format!("{s}"));
    }
    restart_loop(state.clone(), container_ref.clone(), edges.to_vec());
}

/// Reheat the simulation (e.g. after a drag release).
pub fn reheat(state: &Rc<RefCell<SimState>>, container_ref: &NodeRef, edges: &[(usize, usize)]) {
    let mut st = state.borrow_mut();
    st.temperature = REHEAT_TEMP;
    st.settled = false;
    drop(st);
    // Restart the animation loop if it had settled
    restart_loop(state.clone(), container_ref.clone(), edges.to_vec());
}

/// Pin a node to a position (for dragging). Records the offset between cursor and node center.
/// Also reheats and restarts the loop so the dragged node renders during movement.
pub fn pin_node(state: &Rc<RefCell<SimState>>, index: usize, cursor_x: f64, cursor_y: f64, container_ref: &NodeRef, edges: &[(usize, usize)]) {
    {
        let mut st = state.borrow_mut();
        if index < st.particles.len() {
            st.drag_offset_x = st.particles[index].x - cursor_x;
            st.drag_offset_y = st.particles[index].y - cursor_y;
            st.dragging = Some(index);
            st.particles[index].pinned = true;
            st.particles[index].vx = 0.0;
            st.particles[index].vy = 0.0;
            st.temperature = REHEAT_TEMP;
            st.settled = false;
        }
    }
    restart_loop(state.clone(), container_ref.clone(), edges.to_vec());
}

/// Move a pinned node, applying the drag offset so it doesn't snap.
pub fn move_node(state: &Rc<RefCell<SimState>>, index: usize, cursor_x: f64, cursor_y: f64) {
    let mut st = state.borrow_mut();
    if index < st.particles.len() {
        st.particles[index].x = cursor_x + st.drag_offset_x;
        st.particles[index].y = cursor_y + st.drag_offset_y;
    }
}

/// Unpin a node (drag end).
pub fn unpin_node(state: &Rc<RefCell<SimState>>, index: usize) {
    let mut st = state.borrow_mut();
    if index < st.particles.len() {
        st.particles[index].pinned = false;
        st.dragging = None;
    }
}

fn restart_loop(state: Rc<RefCell<SimState>>, container_ref: NodeRef, edges: Vec<(usize, usize)>) {
    // Don't restart if already running (has a handle and not settled)
    {
        let st = state.borrow();
        if st._handle.is_some() && !st.settled {
            return;
        }
    }

    let tick: TickFn = Rc::new(RefCell::new(None));
    let tick_for_closure = tick.clone();
    let state_for_tick = state.clone();

    *tick.borrow_mut() = Some(Box::new(move || {
        if state_for_tick.borrow().cancelled {
            return;
        }

        {
            let mut st = state_for_tick.borrow_mut();
            force_step(&mut st);
            if st.settled {
                st._handle = None;
                return;
            }
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
}

/// Start the force-directed simulation.
pub fn start_simulation(
    container_ref: NodeRef,
    node_count: usize,
    width: f64,
    height: f64,
    center_x: f64,
    center_y: f64,
    edges: Vec<(usize, usize)>,
) -> Rc<RefCell<SimState>> {
    let cx = center_x;
    let cy = center_y;
    let scale = width.min(height) / REF_SIZE;
    let radius = width.min(height) * 0.3;

    let particles: Vec<Particle> = (0..node_count)
        .map(|i| {
            let (x, y) = initial_position(i, node_count, cx, cy, radius);
            Particle {
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                pinned: false,
            }
        })
        .collect();

    // Set CSS scale variable on the container
    if let Some(el) = container_ref.cast::<HtmlElement>() {
        let _ = el.style().set_property("--node-scale", &format!("{scale}"));
    }

    let state = Rc::new(RefCell::new(SimState {
        particles,
        edges: edges.clone(),
        width,
        height,
        center_x,
        center_y,
        scale,
        temperature: INITIAL_TEMP,
        cancelled: false,
        settled: false,
        dragging: None,
        drag_offset_x: 0.0,
        drag_offset_y: 0.0,
        _handle: None,
    }));

    restart_loop(state.clone(), container_ref, edges);

    state
}
