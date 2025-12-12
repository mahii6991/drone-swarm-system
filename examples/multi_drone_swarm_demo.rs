//! Multi-Drone Swarm Intelligence Demo
//!
//! This example demonstrates coordinated multi-drone swarm behavior using
//! bio-inspired optimization algorithms. It shows how multiple drones can
//! work together for formation flying, target tracking, and area coverage.
//!
//! # Features Demonstrated
//! - PSO-based formation control
//! - GWO-inspired hierarchical coordination
//! - Swarm consensus and synchronization
//! - Formation patterns (V-formation, circle, grid)
//!
//! # Running (Simulation Mode - No SITL Required)
//! `cargo run --example multi_drone_swarm_demo --features std`
//!
//! # Running with SITL (requires PX4)
//! 1. Start multiple SITL instances: `./simulation/start_sitl.sh swarm 3`
//! 2. Run: `cargo run --example multi_drone_swarm_demo --features simulation`

use std::f32::consts::PI;
use std::time::Instant;

// Swarm configuration
const NUM_DRONES: usize = 5;
const SIMULATION_STEPS: usize = 100;

/// 3D Position
#[derive(Debug, Clone, Copy, Default)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Position {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Velocity vector
#[derive(Debug, Clone, Copy, Default)]
struct Velocity {
    vx: f32,
    vy: f32,
    vz: f32,
}

impl Velocity {
    fn magnitude(&self) -> f32 {
        (self.vx * self.vx + self.vy * self.vy + self.vz * self.vz).sqrt()
    }

    fn clamp_magnitude(&mut self, max_speed: f32) {
        let mag = self.magnitude();
        if mag > max_speed {
            let scale = max_speed / mag;
            self.vx *= scale;
            self.vy *= scale;
            self.vz *= scale;
        }
    }
}

/// Drone state
#[derive(Debug, Clone)]
struct Drone {
    id: usize,
    position: Position,
    velocity: Velocity,
    target: Position,
    role: DroneRole,
    neighbors: Vec<usize>,
}

/// Drone roles in the swarm (GWO-inspired hierarchy)
#[derive(Debug, Clone, Copy, PartialEq)]
enum DroneRole {
    Alpha, // Leader - best solution
    Beta,  // Second-in-command
    Delta, // Third-in-command
    Omega, // Follower
}

impl Drone {
    fn new(id: usize) -> Self {
        Self {
            id,
            position: Position::default(),
            velocity: Velocity::default(),
            target: Position::default(),
            role: DroneRole::Omega,
            neighbors: Vec::new(),
        }
    }

    fn update_position(&mut self, dt: f32) {
        self.position.x += self.velocity.vx * dt;
        self.position.y += self.velocity.vy * dt;
        self.position.z += self.velocity.vz * dt;
    }
}

/// Swarm formation patterns
#[derive(Debug, Clone, Copy)]
enum Formation {
    VFormation,
    Circle,
    Line,
    Grid,
}

/// Swarm controller using bio-inspired algorithms
struct SwarmController {
    drones: Vec<Drone>,
    formation: Formation,
    center: Position,
    target: Position,
    iteration: usize,
    // PSO parameters
    w: f32,  // Inertia weight
    c1: f32, // Cognitive coefficient
    c2: f32, // Social coefficient
    // GWO parameters
    a: f32, // Linearly decreasing parameter
}

impl SwarmController {
    fn new(num_drones: usize) -> Self {
        let mut drones = Vec::new();
        for i in 0..num_drones {
            drones.push(Drone::new(i));
        }

        Self {
            drones,
            formation: Formation::VFormation,
            center: Position::new(0.0, 0.0, 10.0),
            target: Position::new(50.0, 50.0, 10.0),
            iteration: 0,
            w: 0.7,
            c1: 1.5,
            c2: 1.5,
            a: 2.0,
        }
    }

    /// Initialize drone positions in initial formation
    fn initialize(&mut self) {
        let num_drones = self.drones.len();
        println!("[INIT] Initializing {} drones...", num_drones);

        for (i, drone) in self.drones.iter_mut().enumerate() {
            // Spread drones in initial positions
            let angle = 2.0 * PI * (i as f32) / (num_drones as f32);
            let radius = 5.0;
            drone.position = Position::new(radius * angle.cos(), radius * angle.sin(), 10.0);

            // Assign roles (GWO-inspired hierarchy)
            drone.role = match i {
                0 => DroneRole::Alpha,
                1 => DroneRole::Beta,
                2 => DroneRole::Delta,
                _ => DroneRole::Omega,
            };

            println!(
                "  Drone {}: pos=({:.1}, {:.1}, {:.1}), role={:?}",
                i, drone.position.x, drone.position.y, drone.position.z, drone.role
            );
        }

        // Initialize neighbors (ring topology)
        let n = self.drones.len();
        for i in 0..n {
            let neighbors = vec![
                (i + n - 1) % n, // Left neighbor
                (i + 1) % n,     // Right neighbor
            ];
            self.drones[i].neighbors = neighbors;
        }
    }

    /// Calculate formation positions based on pattern
    fn calculate_formation_positions(&self) -> Vec<Position> {
        let n = self.drones.len();
        let mut positions = Vec::new();

        match self.formation {
            Formation::VFormation => {
                // Classic V-formation (like birds)
                let spacing = 8.0;
                let angle = PI / 6.0; // 30 degrees

                for i in 0..n {
                    let side = if i % 2 == 0 { 1.0 } else { -1.0 };
                    let row = (i / 2) as f32;
                    let offset_x = -row * spacing * angle.cos();
                    let offset_y = side * row * spacing * angle.sin();

                    positions.push(Position::new(
                        self.center.x + offset_x,
                        self.center.y + offset_y,
                        self.center.z,
                    ));
                }
            }
            Formation::Circle => {
                let radius = 15.0;
                for i in 0..n {
                    let angle = 2.0 * PI * (i as f32) / (n as f32);
                    positions.push(Position::new(
                        self.center.x + radius * angle.cos(),
                        self.center.y + radius * angle.sin(),
                        self.center.z,
                    ));
                }
            }
            Formation::Line => {
                let spacing = 10.0;
                let start_x = self.center.x - (n as f32 - 1.0) * spacing / 2.0;
                for i in 0..n {
                    positions.push(Position::new(
                        start_x + (i as f32) * spacing,
                        self.center.y,
                        self.center.z,
                    ));
                }
            }
            Formation::Grid => {
                let cols = ((n as f32).sqrt().ceil()) as usize;
                let spacing = 10.0;
                for i in 0..n {
                    let row = i / cols;
                    let col = i % cols;
                    positions.push(Position::new(
                        self.center.x + (col as f32) * spacing
                            - (cols as f32 - 1.0) * spacing / 2.0,
                        self.center.y + (row as f32) * spacing,
                        self.center.z,
                    ));
                }
            }
        }

        positions
    }

    /// Update swarm using PSO-based formation control
    fn update_pso(&mut self, dt: f32) {
        let formation_positions = self.calculate_formation_positions();

        // Update each drone
        for i in 0..self.drones.len() {
            let target = formation_positions[i];
            let drone = &self.drones[i];

            // PSO velocity update
            let r1 = pseudo_random(self.iteration * 100 + i * 10);
            let r2 = pseudo_random(self.iteration * 100 + i * 10 + 1);

            // Calculate new velocity
            let mut new_vx = self.w * drone.velocity.vx
                + self.c1 * r1 * (target.x - drone.position.x)
                + self.c2 * r2 * (self.center.x - drone.position.x);

            let mut new_vy = self.w * drone.velocity.vy
                + self.c1 * r1 * (target.y - drone.position.y)
                + self.c2 * r2 * (self.center.y - drone.position.y);

            let mut new_vz =
                self.w * drone.velocity.vz + self.c1 * r1 * (target.z - drone.position.z);

            // Clamp velocity
            let max_speed = 5.0;
            let speed = (new_vx * new_vx + new_vy * new_vy + new_vz * new_vz).sqrt();
            if speed > max_speed {
                let scale = max_speed / speed;
                new_vx *= scale;
                new_vy *= scale;
                new_vz *= scale;
            }

            // Apply update
            let drone = &mut self.drones[i];
            drone.velocity.vx = new_vx;
            drone.velocity.vy = new_vy;
            drone.velocity.vz = new_vz;
            drone.target = target;
            drone.update_position(dt);
        }
    }

    /// Update swarm using GWO-inspired coordination
    fn update_gwo(&mut self, dt: f32) {
        // Linearly decrease 'a' parameter (exploration to exploitation)
        self.a = 2.0 - (self.iteration as f32) * (2.0 / SIMULATION_STEPS as f32);

        // Get leader positions (alpha, beta, delta)
        let alpha_pos = self
            .drones
            .iter()
            .find(|d| d.role == DroneRole::Alpha)
            .map(|d| d.position)
            .unwrap_or(self.center);

        let beta_pos = self
            .drones
            .iter()
            .find(|d| d.role == DroneRole::Beta)
            .map(|d| d.position)
            .unwrap_or(self.center);

        let delta_pos = self
            .drones
            .iter()
            .find(|d| d.role == DroneRole::Delta)
            .map(|d| d.position)
            .unwrap_or(self.center);

        // Update each omega wolf position based on leaders
        for i in 0..self.drones.len() {
            if self.drones[i].role == DroneRole::Omega {
                let r1 = pseudo_random(self.iteration * 100 + i * 10);
                let r2 = pseudo_random(self.iteration * 100 + i * 10 + 1);

                // GWO position update (simplified)
                let a_vec = 2.0 * self.a * r1 - self.a;
                let c_vec = 2.0 * r2;

                let drone = &self.drones[i];

                // Calculate attraction to leaders
                let d_alpha = (c_vec * alpha_pos.x - drone.position.x).abs();
                let d_beta = (c_vec * beta_pos.x - drone.position.x).abs();
                let d_delta = (c_vec * delta_pos.x - drone.position.x).abs();

                let x1 = alpha_pos.x - a_vec * d_alpha;
                let x2 = beta_pos.x - a_vec * d_beta;
                let x3 = delta_pos.x - a_vec * d_delta;

                let new_x = (x1 + x2 + x3) / 3.0;

                // Similarly for y
                let d_alpha_y = (c_vec * alpha_pos.y - drone.position.y).abs();
                let d_beta_y = (c_vec * beta_pos.y - drone.position.y).abs();
                let d_delta_y = (c_vec * delta_pos.y - drone.position.y).abs();

                let y1 = alpha_pos.y - a_vec * d_alpha_y;
                let y2 = beta_pos.y - a_vec * d_beta_y;
                let y3 = delta_pos.y - a_vec * d_delta_y;

                let new_y = (y1 + y2 + y3) / 3.0;

                // Update velocity towards new position
                let drone = &mut self.drones[i];
                drone.velocity.vx = (new_x - drone.position.x) * 0.5;
                drone.velocity.vy = (new_y - drone.position.y) * 0.5;
                drone.velocity.clamp_magnitude(5.0);
                drone.update_position(dt);
            }
        }

        // Move leaders toward target
        for drone in self.drones.iter_mut() {
            if drone.role != DroneRole::Omega {
                let speed = match drone.role {
                    DroneRole::Alpha => 3.0,
                    DroneRole::Beta => 2.5,
                    DroneRole::Delta => 2.0,
                    _ => 1.0,
                };

                let dx = self.target.x - drone.position.x;
                let dy = self.target.y - drone.position.y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > 1.0 {
                    drone.velocity.vx = (dx / dist) * speed;
                    drone.velocity.vy = (dy / dist) * speed;
                    drone.update_position(dt);
                }
            }
        }
    }

    /// Calculate swarm metrics
    fn calculate_metrics(&self) -> SwarmMetrics {
        // Calculate center of mass
        let mut center = Position::default();
        for drone in &self.drones {
            center.x += drone.position.x;
            center.y += drone.position.y;
            center.z += drone.position.z;
        }
        let n = self.drones.len() as f32;
        center.x /= n;
        center.y /= n;
        center.z /= n;

        // Calculate spread (max distance from center)
        let mut max_spread = 0.0f32;
        let mut min_separation = f32::INFINITY;

        for (i, drone) in self.drones.iter().enumerate() {
            let dist = drone.position.distance_to(&center);
            max_spread = max_spread.max(dist);

            // Check separation from other drones
            for (j, other) in self.drones.iter().enumerate() {
                if i != j {
                    let sep = drone.position.distance_to(&other.position);
                    min_separation = min_separation.min(sep);
                }
            }
        }

        // Calculate formation error
        let formation_positions = self.calculate_formation_positions();
        let mut formation_error = 0.0;
        for (drone, target) in self.drones.iter().zip(formation_positions.iter()) {
            formation_error += drone.position.distance_to(target);
        }
        formation_error /= self.drones.len() as f32;

        SwarmMetrics {
            center,
            spread: max_spread,
            min_separation,
            formation_error,
            avg_velocity: self
                .drones
                .iter()
                .map(|d| d.velocity.magnitude())
                .sum::<f32>()
                / n,
        }
    }

    /// Run simulation step
    fn step(&mut self, dt: f32, algorithm: &str) {
        match algorithm {
            "pso" => self.update_pso(dt),
            "gwo" => self.update_gwo(dt),
            _ => self.update_pso(dt),
        }
        self.iteration += 1;
    }

    /// Move swarm center toward target
    fn move_center_toward_target(&mut self, speed: f32) {
        let dx = self.target.x - self.center.x;
        let dy = self.target.y - self.center.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > 1.0 {
            self.center.x += (dx / dist) * speed;
            self.center.y += (dy / dist) * speed;
        }
    }

    /// Set new formation pattern
    fn set_formation(&mut self, formation: Formation) {
        self.formation = formation;
        println!("[FORMATION] Switching to {:?}", formation);
    }
}

/// Swarm metrics
#[derive(Debug)]
struct SwarmMetrics {
    center: Position,
    spread: f32,
    min_separation: f32,
    formation_error: f32,
    avg_velocity: f32,
}

/// Pseudo-random number generator
fn pseudo_random(seed: usize) -> f32 {
    let a = 1_103_515_245_u64;
    let c = 12_345_u64;
    let m = 2_147_483_648_u64;
    let x = ((a.wrapping_mul(seed as u64).wrapping_add(c)) % m) as f32;
    x / m as f32
}

fn main() {
    println!("=== Multi-Drone Swarm Intelligence Demo ===\n");
    println!("Demonstrating coordinated swarm behavior using:");
    println!("  - PSO (Particle Swarm Optimization) for formation control");
    println!("  - GWO (Grey Wolf Optimizer) for hierarchical coordination");
    println!("\nSwarm size: {} drones\n", NUM_DRONES);

    let start_time = Instant::now();

    // Create swarm controller
    let mut swarm = SwarmController::new(NUM_DRONES);
    swarm.initialize();

    println!("\n--- Phase 1: V-Formation Flight (PSO) ---\n");
    swarm.set_formation(Formation::VFormation);

    // Simulate PSO-based formation
    for step in 0..30 {
        swarm.step(0.1, "pso");
        swarm.move_center_toward_target(0.5);

        if step % 10 == 0 {
            let metrics = swarm.calculate_metrics();
            println!(
                "[Step {:3}] Center: ({:5.1}, {:5.1}) | Spread: {:5.1}m | Formation Error: {:5.2}m | Avg Speed: {:4.2} m/s",
                step, metrics.center.x, metrics.center.y, metrics.spread, metrics.formation_error, metrics.avg_velocity
            );
        }
    }

    println!("\n--- Phase 2: Circle Formation (PSO) ---\n");
    swarm.set_formation(Formation::Circle);
    swarm.target = Position::new(30.0, 30.0, 10.0);

    for step in 30..60 {
        swarm.step(0.1, "pso");
        swarm.move_center_toward_target(0.3);

        if step % 10 == 0 {
            let metrics = swarm.calculate_metrics();
            println!(
                "[Step {:3}] Center: ({:5.1}, {:5.1}) | Spread: {:5.1}m | Formation Error: {:5.2}m | Min Sep: {:4.1}m",
                step, metrics.center.x, metrics.center.y, metrics.spread, metrics.formation_error, metrics.min_separation
            );
        }
    }

    println!("\n--- Phase 3: Leader-Follower Hunt (GWO) ---\n");
    swarm.target = Position::new(60.0, 60.0, 10.0);

    for step in 60..100 {
        swarm.step(0.1, "gwo");

        if step % 10 == 0 {
            let metrics = swarm.calculate_metrics();
            println!(
                "[Step {:3}] Center: ({:5.1}, {:5.1}) | Spread: {:5.1}m | a={:.2} (exploration→exploitation)",
                step, metrics.center.x, metrics.center.y, metrics.spread, swarm.a
            );

            // Show leader positions
            for drone in &swarm.drones {
                if drone.role != DroneRole::Omega {
                    println!(
                        "   {:?}: ({:5.1}, {:5.1})",
                        drone.role, drone.position.x, drone.position.y
                    );
                }
            }
        }
    }

    // Final metrics
    let final_metrics = swarm.calculate_metrics();
    let elapsed = start_time.elapsed();

    println!("\n=== Simulation Summary ===");
    println!("Total runtime: {:.2}ms", elapsed.as_secs_f32() * 1000.0);
    println!("Simulation steps: {}", SIMULATION_STEPS);
    println!("Drones: {}", NUM_DRONES);
    println!("\nFinal State:");
    println!(
        "  Swarm center: ({:.1}, {:.1}, {:.1})",
        final_metrics.center.x, final_metrics.center.y, final_metrics.center.z
    );
    println!("  Swarm spread: {:.1}m", final_metrics.spread);
    println!(
        "  Min drone separation: {:.1}m",
        final_metrics.min_separation
    );
    println!("  Formation error: {:.2}m", final_metrics.formation_error);

    println!("\nDrone Final Positions:");
    for drone in &swarm.drones {
        println!(
            "  Drone {} ({:?}): ({:6.1}, {:6.1}, {:6.1})",
            drone.id, drone.role, drone.position.x, drone.position.y, drone.position.z
        );
    }

    println!("\n[OK] Multi-drone swarm demo completed!");
    println!("\nAlgorithms demonstrated:");
    println!("  PSO: Formation control with cognitive + social learning");
    println!("  GWO: Hierarchical leader-follower coordination");
    println!("\nNext steps:");
    println!("  - Connect to PX4 SITL for realistic drone physics");
    println!("  - Run: cargo run --example swarm_sitl --features \"simulation tokio\"");
}
