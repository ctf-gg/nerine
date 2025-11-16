use axum::{extract::State as StateE, routing::get, Json, Router};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

use crate::{Result, State};

// TODO(aiden): in the future it would be really cool
// if on init of backend it could auto build a default
// frontend.

// also eventually it should go to an event.toml somewhere
#[derive(Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub description: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    #[serde(default)]
    pub divisions: HashMap<String, String>,
}

impl Event {
    // NOTE(ani): ought to be AsRef<Path> but I don't care
    pub fn read_from_path(path: &str) -> eyre::Result<Self> {
        let cfg = fs::read_to_string(path)?;
        Ok(toml::from_str(&cfg)?)
    }
}

// ported from https://github.com/blueset/CTFd/blob/luna/CTFd/plugins/dynamic_challenges/decay.py#L72
pub fn point_formula(points_min: i32, points_max: i32, solves: i32) -> i32 {
    // note(jayden): ctfd has this configurable per chall, but here im hardcoding to 60 (same as used in sekaictf)
    let decay = 100.0;
    let gradient = 10.0;

    let _min = 1.0 + ((gradient - 1.0) / decay);
    let x = 1.0 + ((gradient - 1.0) / decay) * solves as f64;
    let ratio = (x / _min).ln() / (gradient / _min).ln();
    let raw_score = (points_max as f64 - (points_max - points_min) as f64 * ratio).ceil();
    let value = points_min.max(points_max.min(raw_score as i32));

    value
}
/* web routes
NOTE(ani): keep it small or move into a separate file */

async fn event_route(StateE(state): StateE<State>) -> Result<Json<Event>> {
    Ok(Json(state.event.clone()))
}

pub fn router() -> Router<crate::State> {
    Router::new().route("/", get(event_route))
}
