//! PSO Formation Control with SITL
//!
//! This example demonstrates using PSO-style waypoint generation to control
//! drone formation in PX4 SITL simulation.
//!
//! # Running
//! 1. Start PX4 SITL: `cd ~/PX4-Autopilot/build/px4_sitl_default && ./bin/px4 -d`
//! 2. Run this example: `cargo run --example pso_sitl_formation --features simulation`

use std::f32::consts::PI;
use std::time::{Duration, Instant};

use mavlink::common::{MavCmd, MavMessage, MavModeFlag, PositionTargetTypemask, COMMAND_LONG_DATA};
use mavlink::MavHeader;

const SITL_ADDRESS: &str = "udpout:127.0.0.1:14540";

/// Current drone state from telemetry
#[derive(Debug, Clone, Default)]
struct DroneState {
    position: [f32; 3],
    velocity: [f32; 3],
    armed: bool,
}

/// Send MAVLink command
fn send_command<M: mavlink::Message>(
    conn: &dyn mavlink::MavConnection<M>,
    target_system: u8,
    target_component: u8,
    cmd: MavCmd,
    params: [f32; 7],
) where
    MavMessage: Into<M>,
{
    let header = MavHeader {
        system_id: 255,
        component_id: 0,
        sequence: 0,
    };

    let msg = MavMessage::COMMAND_LONG(COMMAND_LONG_DATA {
        target_system,
        target_component,
        command: cmd,
        confirmation: 0,
        param1: params[0],
        param2: params[1],
        param3: params[2],
        param4: params[3],
        param5: params[4],
        param6: params[5],
        param7: params[6],
    });

    let _ = conn.send(&header, &msg.into());
}

/// Generate formation waypoints in a circle pattern
fn generate_circle_waypoints(num_points: usize, radius: f32, altitude: f32) -> Vec<[f32; 3]> {
    let mut waypoints = Vec::new();

    for i in 0..num_points {
        let angle = 2.0 * PI * (i as f32) / (num_points as f32);
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let z = -altitude; // NED frame: negative Z is up
        waypoints.push([x, y, z]);
    }

    waypoints
}

fn main() {
    println!("=== PSO Formation Control with SITL ===\n");

    // Connect to SITL
    println!("Connecting to SITL at {}...", SITL_ADDRESS);
    let conn = match mavlink::connect::<MavMessage>(SITL_ADDRESS) {
        Ok(c) => {
            println!("[OK] Connected!");
            c
        }
        Err(e) => {
            eprintln!("[ERROR] Connection failed: {}", e);
            eprintln!("Make sure PX4 SITL is running:");
            eprintln!("  cd ~/PX4-Autopilot/build/px4_sitl_default && ./bin/px4 -d");
            return;
        }
    };

    // Generate formation waypoints (circle pattern)
    println!("\n[FORMATION] Generating circle waypoints...");
    let waypoints = generate_circle_waypoints(8, 15.0, 10.0);
    println!("[FORMATION] Generated {} waypoints:", waypoints.len());
    for (i, wp) in waypoints.iter().enumerate() {
        println!("  WP{}: ({:6.1}, {:6.1}, {:6.1})", i, wp[0], wp[1], wp[2]);
    }

    // State tracking
    let mut state = DroneState::default();
    let mut current_waypoint = 0;
    let start_time = Instant::now();
    let mut last_wp_time = Instant::now();
    let mut position_count = 0;
    let mut waypoints_reached = 0;

    // Arm the drone
    println!("\n[CMD] Attempting to arm...");
    send_command(
        conn.as_ref(),
        1,
        1,
        MavCmd::MAV_CMD_COMPONENT_ARM_DISARM,
        [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    );

    println!("\n[NAV] Starting formation flight demonstration...");
    println!(
        "Will fly through {} waypoints in circle formation\n",
        waypoints.len()
    );

    // Main control loop
    loop {
        // Check timeout (60 seconds)
        if start_time.elapsed() > Duration::from_secs(60) {
            println!("\n[TIMEOUT] 60 second limit reached");
            break;
        }

        // Receive telemetry
        match conn.recv() {
            Ok((_header, msg)) => {
                match msg {
                    MavMessage::LOCAL_POSITION_NED(pos) => {
                        state.position = [pos.x, pos.y, pos.z];
                        state.velocity = [pos.vx, pos.vy, pos.vz];
                        position_count += 1;

                        // Print position occasionally
                        if position_count % 100 == 0 {
                            let target = &waypoints[current_waypoint];
                            let dx = pos.x - target[0];
                            let dy = pos.y - target[1];
                            let dz = pos.z - target[2];
                            let distance = (dx * dx + dy * dy + dz * dz).sqrt();

                            println!(
                                "[POS] ({:6.1}, {:6.1}, {:6.1}) → WP{} ({:6.1}, {:6.1}, {:6.1}) dist={:.1}m",
                                pos.x, pos.y, pos.z,
                                current_waypoint,
                                target[0], target[1], target[2],
                                distance
                            );
                        }

                        // Check if reached current waypoint (within 3m)
                        let target = &waypoints[current_waypoint];
                        let dx = pos.x - target[0];
                        let dy = pos.y - target[1];
                        let distance_2d = (dx * dx + dy * dy).sqrt();

                        if distance_2d < 3.0 && last_wp_time.elapsed() > Duration::from_secs(2) {
                            waypoints_reached += 1;
                            println!(
                                "[NAV] ✓ Reached WP{} (distance: {:.1}m) [{}/{}]",
                                current_waypoint,
                                distance_2d,
                                waypoints_reached,
                                waypoints.len()
                            );
                            current_waypoint = (current_waypoint + 1) % waypoints.len();
                            last_wp_time = Instant::now();

                            if current_waypoint == 0 && waypoints_reached >= waypoints.len() {
                                println!("[NAV] Completed full formation circle!");
                                break;
                            }
                        }
                    }
                    MavMessage::HEARTBEAT(hb) => {
                        let was_armed = state.armed;
                        state.armed = (hb.base_mode.bits()
                            & MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED.bits())
                            != 0;
                        if state.armed && !was_armed {
                            println!("[STATUS] Drone ARMED");
                        }
                    }
                    MavMessage::STATUSTEXT(text) => {
                        let msg_text: String = text
                            .text
                            .iter()
                            .take_while(|&&c| c != 0)
                            .map(|&c| c as char)
                            .collect();
                        if !msg_text.is_empty() {
                            println!("[PX4] {}", msg_text);
                        }
                    }
                    _ => {}
                }
            }
            Err(_) => {
                // Receive error, continue
            }
        }

        // Send position setpoint continuously (for offboard control)
        let target = &waypoints[current_waypoint];
        let header = MavHeader {
            system_id: 255,
            component_id: 0,
            sequence: 0,
        };

        let setpoint = MavMessage::SET_POSITION_TARGET_LOCAL_NED(
            mavlink::common::SET_POSITION_TARGET_LOCAL_NED_DATA {
                time_boot_ms: 0,
                target_system: 1,
                target_component: 1,
                coordinate_frame: mavlink::common::MavFrame::MAV_FRAME_LOCAL_NED,
                type_mask: PositionTargetTypemask::empty(), // Use all fields
                x: target[0],
                y: target[1],
                z: target[2],
                vx: 0.0,
                vy: 0.0,
                vz: 0.0,
                afx: 0.0,
                afy: 0.0,
                afz: 0.0,
                yaw: 0.0,
                yaw_rate: 0.0,
            },
        );

        let _ = conn.send(&header, &setpoint);
    }

    println!("\n=== Summary ===");
    println!(
        "Waypoints reached: {}/{}",
        waypoints_reached,
        waypoints.len()
    );
    println!("Runtime: {:.1}s", start_time.elapsed().as_secs_f32());
    println!("Position updates: {}", position_count);
    println!("\n[OK] PSO formation demo completed!");
}
