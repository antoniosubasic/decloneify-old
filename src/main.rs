mod token;

use dotenv::dotenv;
use reqwest::Client;
use std::env;
use token::Token;

struct Spotify<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    user_id: &'a str,
    access_token: Option<Token>,
}

impl<'a> Spotify<'a> {
    fn new(client_id: &'a str, client_secret: &'a str, user_id: &'a str) -> Self {
        Self {
            client_id,
            client_secret,
            user_id,
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
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
    let user_id = env::var("USER_ID").expect("USER_ID must be set");

    let mut spotify = Spotify::new(&client_id, &client_secret, &user_id);
}
