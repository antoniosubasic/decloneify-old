use serde::Deserialize;

#[derive(Deserialize)]
pub struct Tracks {
    pub href: String,
    pub total: u64,
}

#[derive(Deserialize)]
pub struct Playlist {
    pub description: String,
    pub id: String,
    pub name: String,
    pub tracks: Tracks,
}

#[derive(Deserialize)]
pub struct PlaylistResponse {
    pub items: Vec<Playlist>,
}
