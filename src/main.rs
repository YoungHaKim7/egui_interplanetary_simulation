use eframe::{self, App, Frame, egui};
use egui::{Color32, Pos2, Rect};
use nalgebra::Vector2;
use rand::Rng;

const G: f32 = 6.67430e-5; // Gravitational constant

struct CelestialBody {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    mass: f32,
    radius: f32,
    color: Color32,
}

impl CelestialBody {
    fn new(pos: Vector2<f32>, mass: f32, color: Color32) -> Self {
        Self {
            pos,
            vel: Vector2::zeros(),
            mass,
            radius: (mass / std::f32::consts::PI).sqrt() / 2.0,
            color,
        }
    }

    // fn apply_gravity(&mut self, other: &CelestialBody) {
    //     let dir = other.pos - self.pos;
    //     let dist_sq = dir.norm_squared();
    //     if dist_sq > (self.radius + other.radius).powi(2) {
    //         let force_mag = G * self.mass * other.mass / dist_sq;
    //         let force = dir.normalize() * force_mag;
    //         self.vel += force / self.mass;
    //     }
    // }

    fn apply_gravity(&mut self, other: &CelestialBody) {
        let dir = other.pos - self.pos;
        let dist_sq = dir.norm_squared();
        let dist = dist_sq.sqrt();

        if dist_sq > (self.radius + other.radius).powi(2) {
            let force_mag = G * self.mass * other.mass / dist_sq;
            let force = dir.normalize() * force_mag;
            self.vel += force / self.mass;

            // If close to a big mass, give extra tangential velocity to "orbit"
            if dist < 150.0 && other.mass > self.mass * 5.0 {
                let tangential = Vector2::new(-dir.y, dir.x).normalize();
                self.vel += tangential * 0.05;
            }
        }
    }
    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
    }
}

struct InterplanetarySimulation {
    bodies: Vec<CelestialBody>,
    camera_pos: Pos2,
    zoom: f32,
}

impl Default for InterplanetarySimulation {
    fn default() -> Self {
        let mut bodies = Vec::new();
        // Sun
        bodies.push(CelestialBody::new(
            Vector2::new(400.0, 300.0),
            10000.0,
            Color32::YELLOW,
        ));
        // Earth
        bodies.push(CelestialBody::new(
            Vector2::new(500.0, 300.0),
            100.0,
            Color32::from_rgb(0, 128, 255),
        ));
        bodies[1].vel.y = 80.0;

        let mut bodies = Vec::new();
        let mut rng = rand::rng();
        // Asteroids
        for _ in 0..200 {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = rng.random_range(150.0..350.0);
            let pos = Vector2::new(
                400.0 + distance * angle.cos(),
                300.0 + distance * angle.sin(),
            );
            let mass = rng.random_range(1.0..5.0);
            let color = Color32::GRAY;
            let mut asteroid = CelestialBody::new(pos, mass, color);

            let to_center = Vector2::new(400.0, 300.0) - pos;
            let tangential = Vector2::new(-to_center.y, to_center.x).normalize();
            asteroid.vel = tangential * rng.random_range(10.0..30.0);

            bodies.push(asteroid);
        }

        Self {
            bodies,
            camera_pos: Pos2::new(400.0, 300.0),
            zoom: 1.0,
        }
    }
}

impl App for InterplanetarySimulation {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, _response) =
                ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

            // Handle camera movement
            if ui.input(|i| i.pointer.primary_down()) {
                self.camera_pos -= ui.input(|i| i.pointer.delta());
            }
            // self.zoom *= (1.0 + ui.input(|i| i.raw.scroll_delta.y) / 200.0).max(0.1);

            self.zoom *= f32::max(0.1, 1.0 + ui.input(|i| i.raw_scroll_delta.y) / 200.0);

            let painter = ui.painter();
            painter.rect_filled(rect, 0.0, Color32::BLACK);

            // Simulation logic
            let dt = ui.input(|i| i.stable_dt);
            for i in 0..self.bodies.len() {
                for j in 0..self.bodies.len() {
                    if i == j {
                        continue;
                    }
                    let other = unsafe { &*(self.bodies.get(j).unwrap() as *const _) };
                    self.bodies[i].apply_gravity(other);
                }
            }
            let camera_pos = self.camera_pos;
            let zoom = self.zoom;
            let center = rect.center();

            for body in &mut self.bodies {
                body.update(dt);
                let screen_vec = (body.pos - Vector2::new(camera_pos.x, camera_pos.y)) * zoom;
                let screen_pos = Pos2::new(center.x + screen_vec.x, center.y + screen_vec.y);
                painter.circle_filled(screen_pos, body.radius * zoom, body.color);
            }
            // for body in &mut self.bodies {
            //     body.update(dt);
            //     let screen_pos = self.world_to_screen(body.pos, rect);
            //     painter.circle_filled(screen_pos, body.radius * self.zoom, body.color);
            // }

            // UI Controls
            egui::Window::new("Controls").show(ctx, |ui| {
                if ui.button("Reset").clicked() {
                    *self = Self::default();
                }
                if ui.button("Add Planet").clicked() {
                    let mut rng = rand::rng();
                    let pos =
                        Vector2::new(rng.random_range(0.0..800.0), rng.random_range(0.0..600.0));
                    let mass = rng.random_range(1500.0..2200.0);
                    let color = Color32::from_rgb(
                        rng.random_range(0..255),
                        rng.random_range(0..255),
                        rng.random_range(0..255),
                    );
                    self.bodies.push(CelestialBody::new(pos, mass, color));
                }
            });
            ui.ctx().request_repaint();
        });
    }
}

impl InterplanetarySimulation {
    fn world_to_screen(&self, world_pos: Vector2<f32>, rect: Rect) -> Pos2 {
        let center = rect.center();
        let screen_vec =
            (world_pos - Vector2::new(self.camera_pos.x, self.camera_pos.y)) * self.zoom;
        Pos2::new(center.x + screen_vec.x, center.y + screen_vec.y)
    }
    // fn world_to_screen(&self, world_pos: Vector2<f32>, rect: Rect) -> Pos2 {

    //     let center = rect.center();
    //     (world_pos - Vector2::new(self.camera_pos.x, self.camera_pos.y)) * self.zoom
    //         + Vector2::new(center.x, center.y)
    // }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 1200.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Interplanetary Simulation",
        options,
        Box::new(|_cc| Ok(Box::<InterplanetarySimulation>::default())),
    )
    .unwrap();
}
