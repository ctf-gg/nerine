use std::{cmp::max, fs, sync::LazyLock};
use serde::Deserialize;
use toml;

use chrono::NaiveDateTime;

// TODO(aiden): in the future it would be really cool
// if on init of backend it could auto build a default
// frontend.

// also eventually it should go to an event.toml somewhere
#[derive(Clone, Deserialize)]
pub struct Event {
    pub name: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}

// TODO(aiden): make path configurable
pub static EVENT: LazyLock<Event> = LazyLock::new(|| {
    let event_cfg = fs::read_to_string("event.toml").expect("Expected event.toml in working directory");
    toml::from_str(&event_cfg).unwrap()
});

pub fn point_formula(
    points_min: i32, points_max: i32, solves: i32,
) -> i32 {
    return max(points_min, points_max - (points_max - points_min) * solves / 20)
}