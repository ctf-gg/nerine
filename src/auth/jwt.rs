#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize,
    team_id: String,
}