# Rusty Drones
Drone for the Advanced Programming university's project.

## Importing the project
Add to your `cargo.toml` the line:
```toml
rusty_drones = { git = "https://github.com/matteo-dinoia/rusty-drones.git", branch = "main" }
```
And obviously it is required to import the class repo for example like this:
```toml
wg_2024 = { git = "https://github.com/WGL-2024/WGL_repo_2024.git" }
```
The code may change, if you want to have the latest version don't forget to run `cargo update` periodically.

## Using the drone
```rust
use rusty_drones::RustyDrone;
use wg_2024::drone::Drone;

fn main() {
    /* ... */
    RustyDrone::new(/* add missing arguments */);
    /* ... */
}
```


## Extra test usable also for other drones
Coming soon.

## Support
You can contact us:
1. Telegram Bot [@RustyDronesBot](https://t.me/RustyDronesBot), we will quickly respond and fix any of your problems
2. Create an issue on this GitHub repo
