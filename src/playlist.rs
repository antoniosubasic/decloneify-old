use serde::Deserialize;

#[derive(Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct PlaylistResponse {
    pub items: Vec<Playlist>,
}
