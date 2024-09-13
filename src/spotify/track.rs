use serde::Deserialize;

#[derive(Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct TrackData {
    pub track: Option<Track>,
}

#[derive(Deserialize)]
pub struct TrackResponse {
    pub items: Vec<TrackData>,
    pub next: Option<String>,
}
