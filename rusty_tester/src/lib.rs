pub mod flood;
pub mod general;
pub mod utils;

pub use utils::data;

use std::time::Duration;
const TIMEOUT: Duration = Duration::from_millis(50);
