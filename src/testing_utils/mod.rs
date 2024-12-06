mod drone_options;

pub use drone_options::DroneOptions;
use crate::drone::MyDrone;

pub fn test_initialization() -> (DroneOptions, MyDrone){
    let options = DroneOptions::new();
    let drone = options.create_drone();

    (options, drone)
}