use std::{cmp::max, fs};
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

impl Event {
    // NOTE(ani): ought to be AsRef<Path> but I don't care
    pub fn read_from_path(path: &str) -> eyre::Result<Self> {
        let cfg = fs::read_to_string(path)?;
        Ok(toml::from_str(&cfg)?)
    }
}

pub fn point_formula(
    points_min: i32, points_max: i32, solves: i32,
) -> i32 {
    return max(points_min, points_max - (points_max - points_min) * solves / 20)
}
