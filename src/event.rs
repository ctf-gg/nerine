use std::{cmp::max, str::FromStr, sync::LazyLock};

use chrono::NaiveDateTime;

// TODO(aiden): in the future it would be really cool
// if on init of backend it could auto build a default
// frontend.

// also eventually it should go to an event.toml somewhere
#[derive(Clone)]
pub struct Event {
    pub name: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}

pub static EVENT: LazyLock<Event> = LazyLock::new(|| Event {
    name: String::from("smileyCTF"),
    start_time: NaiveDateTime::from_str("2025-04-25T22:36:51.356942").unwrap(),
    end_time: NaiveDateTime::from_str("2025-04-27T22:36:51.356942").unwrap(),
});

pub fn point_formula(
    points_min: i64, points_max: i64, solves: i64,
) -> i64 {
    return max(points_min, points_max - (points_max - points_min) * solves / 20)
}
