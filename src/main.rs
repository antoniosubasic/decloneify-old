mod playlist;
mod token;
mod track;

use dotenv::dotenv;
use playlist::{Playlist, PlaylistResponse};
use reqwest::Client;
use std::env;
use token::Token;
use track::{Track, TrackResponse};

struct Spotify<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    access_token: Option<Token>,
}

impl<'a> Spotify<'a> {
    fn new(client_id: &'a str, client_secret: &'a str) -> Self {
        Self {
            client_id,
            client_secret,
            access_token: None,
        }
    }

    async fn request_access_token(&mut self) -> Result<(), String> {
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

    async fn get_user_playlists(&mut self, user_id: &str) -> Result<Vec<Playlist>, String> {
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

    async fn get_tracks(
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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
    let user_id = env::var("USER_ID").expect("USER_ID must be set");

    let mut spotify = Spotify::new(&client_id, &client_secret);

    match spotify.get_user_playlists(&user_id).await {
        Ok(playlists) => {
            for playlist in playlists {
                println!("{} ({})", playlist.name, playlist.id);
                match spotify.get_tracks(&playlist.id, None).await {
                    Ok(tracks) => {
                        for track in tracks {
                            println!("{}", track.name);
                        }
                        println!();
                    }
                    Err(e) => eprintln!("{}", e),
                }
            }
        }
        Err(e) => eprintln!("{}", e),
    }
}
