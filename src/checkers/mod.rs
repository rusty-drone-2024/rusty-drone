#![cfg(test)]

use std::time::Duration;

mod integrations;
mod wgl_test;
mod flood;

const TIMEOUT: Duration = Duration::from_millis(100);