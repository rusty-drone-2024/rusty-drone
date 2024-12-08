#![cfg(test)]
#![allow(dead_code)]

pub mod data;
mod drone_options;
mod network_initializer;

use crate::drone::RustyDrone;
pub use drone_options::DroneOptions;
pub use network_initializer::Network;

use wg_2024::network::NodeId;

pub fn test_initialization() -> (DroneOptions, RustyDrone) {
    let options = DroneOptions::new();
    let drone = options.create_drone(1, 0.0);

    (options, drone)
}

pub fn test_initialization_with_value(id: NodeId, pdr: f32) -> (DroneOptions, RustyDrone) {
    let options = DroneOptions::new();
    let drone = options.create_drone(id, pdr);

    (options, drone)
}

pub fn test_muliple_initialization(amount: usize) -> Vec<(DroneOptions, RustyDrone)> {
    (0..)
        .map(|i| {
            let drone_options = DroneOptions::new();
            let drone = drone_options.create_drone(i, 0.0);

            (drone_options, drone)
        })
        .take(amount)
        .collect::<Vec<_>>()
}
