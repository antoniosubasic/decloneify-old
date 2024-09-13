use crate::spotify::{Playlist, PlaylistResponse, Token, Track, TrackResponse};
use reqwest::Client;

pub struct Spotify<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    access_token: Option<Token>,
}

impl<'a> Spotify<'a> {
    pub fn new(client_id: &'a str, client_secret: &'a str) -> Self {
        Self {
            client_id,
            client_secret,
            access_token: None,
        }
    }

    pub async fn request_access_token(&mut self) -> Result<(), String> {
        if self.access_token.is_some() {
            let token = self.access_token.as_ref().unwrap();
            let now = chrono::Utc::now();

            if now < token.expiration {
                return Ok(());
            }
        }

        let response = Client::new()
            .post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "grant_type=client_credentials&client_id={}&client_secret={}",
                self.client_id, self.client_secret
            ))
            .send()
            .await;

        match response {
            Ok(response) => match response.status().is_success() {
                true => match response.json().await {
                    Ok(token) => {
                        self.access_token = Some(token);
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                },
                false => Err(response.text().await.unwrap()),
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn get_user_playlists(&mut self, user_id: &str) -> Result<Vec<Playlist>, String> {
        self.request_access_token().await?;

        let token = self.access_token.as_ref().unwrap();
        let response = Client::new()
            .get(format!(
                "https://api.spotify.com/v1/users/{}/playlists",
                user_id
            ))
            .header("Authorization", format!("Bearer {}", token.access_token))
            .send()
            .await;

        match response {
            Ok(response) => match response.status().is_success() {
                true => match response.json::<PlaylistResponse>().await {
                    Ok(playlist_response) => Ok(playlist_response.items),
                    Err(e) => Err(e.to_string()),
                },
                false => Err(response.text().await.unwrap()),
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn get_tracks(
        &mut self,
        playlist_id: &str,
        url: Option<String>,
    ) -> Result<Vec<Track>, String> {
        self.request_access_token().await?;

        let url = match url {
            Some(url) => url,
            None => format!(
                "https://api.spotify.com/v1/playlists/{}/tracks",
                playlist_id
            ),
        };

        let token = self.access_token.as_ref().unwrap();
        let response = Client::new()
            .get(url)
            .header("Authorization", format!("Bearer {}", token.access_token))
            .send()
            .await;

        match response {
            Ok(response) => match response.status().is_success() {
                true => match response.json::<TrackResponse>().await {
                    Ok(track_response) => {
                        let mut tracks = track_response
                            .items
                            .into_iter()
                            .filter(|td| td.track.is_some())
                            .map(|td| td.track.unwrap())
                            .collect::<Vec<Track>>();
                        if let Some(next_url) = track_response.next {
                            let next_tracks =
                                Box::pin(self.get_tracks(playlist_id, Some(next_url))).await?;
                            tracks.extend(next_tracks);
                        }
                        Ok(tracks)
                    }
                    Err(e) => Err(e.to_string()),
                },
                false => Err(response.text().await.unwrap()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}
