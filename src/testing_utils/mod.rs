mod drone_options;

use wg_2024::network::NodeId;
pub use drone_options::DroneOptions;
use crate::drone::MyDrone;

pub fn test_initialization() -> (DroneOptions, MyDrone){
    let options = DroneOptions::new();
    let drone = options.create_drone(1, 0.0);

    (options, drone)
}

pub fn test_initialization_with_value(id: NodeId, pdr: f32) -> (DroneOptions, MyDrone){
    let options = DroneOptions::new();
    let drone = options.create_drone(id, pdr);

    (options, drone)
}

pub fn test_muliple_initialization(amount: usize) -> Vec<(DroneOptions, MyDrone)>{
    (0..).map(|i| {
        let drone_options = DroneOptions::new();
        let drone = drone_options.create_drone(i, 0.0);
        
        (drone_options,  drone)
    })
        .take(amount)
        .collect::<Vec<_>>()
}