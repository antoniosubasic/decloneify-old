use serde::Deserialize;

#[derive(Deserialize)]
pub struct Trackmap {
    #[serde(flatten)]
    pub tracknames: std::collections::HashMap<String, IDs>,
}

#[derive(Deserialize)]
pub struct IDs {
    pub ids: Vec<String>,
}
