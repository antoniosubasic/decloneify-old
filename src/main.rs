mod spotify;
mod trackmap;

use dotenv::dotenv;
use std::{collections::HashMap, env, fs};
use trackmap::Trackmap;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
    let user_id = env::var("USER_ID").expect("USER_ID must be set");

    let trackmap_content = fs::read_to_string("trackmap.toml").unwrap_or(String::new());
    let trackmap: Trackmap = toml::from_str(&trackmap_content).expect("invalid trackmap.toml");

    let mut spotify = spotify::Spotify::new(&client_id, &client_secret);

    match spotify.get_user_playlists(&user_id).await {
        Ok(playlists) => {
            for i in 0..playlists.len() {
                println!("=== {} ===", playlists[i].name);
                let mut tracks_by_name: HashMap<String, Vec<String>> = HashMap::new();

                match spotify.get_tracks(&playlists[i].id, None).await {
                    Ok(tracks) => {
                        for track in &tracks {
                            let entry = tracks_by_name
                                .entry(track.name.clone())
                                .or_insert_with(Vec::new);

                            let should_push = match trackmap.tracknames.get(&track.name) {
                                Some(ids) => !ids.ids.iter().any(|id| entry.contains(id)),
                                None => true,
                            };

                            if should_push {
                                entry.push(track.id.clone());
                            }
                        }
                    }
                    Err(e) => eprintln!("{}", e),
                }

                let tracks_by_name: HashMap<String, Vec<String>> = tracks_by_name
                    .iter()
                    .filter(|p| p.1.len() > 1)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                for (name, ids) in &tracks_by_name {
                    print!("{}: ", name);
                    for i in 0..ids.len() {
                        print!("{}{}", ids[i], if i == ids.len() - 1 { "\n" } else { ", " });
                    }
                }

                println!(
                    "{}",
                    if tracks_by_name.len() == 0 {
                        "no duplicates found\n"
                    } else {
                        ""
                    }
                );
            }
        }
        Err(e) => eprintln!("{}", e),
    }
}
