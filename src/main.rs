mod spotify;
mod trackmap;

use colored::Colorize;
use dotenv::dotenv;
use std::{
    collections::HashMap,
    env,
    fs::{self, OpenOptions},
    io::Write,
    process,
};
use trackmap::Trackmap;

fn write_to_env_file(key: &str, value: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(".env")
        .expect("failed to open .env");

    writeln!(file, "{}={}", key, value).expect("failed to write to .env");
}

fn get_env_var(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => {
            print!("{}: ", key);
            std::io::stdout().flush().unwrap();

            let mut val = String::new();
            std::io::stdin().read_line(&mut val).unwrap();
            let val = val.trim().to_string();

            write_to_env_file(key, &val);
            val
        }
    }
}

fn throw_error(e: String) {
    eprintln!("{}", e.red());
    process::exit(1);
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client_id = get_env_var("CLIENT_ID");
    let client_secret = get_env_var("CLIENT_SECRET");
    let user_id = get_env_var("USER_ID");

    let trackmap: Trackmap =
        toml::from_str(&fs::read_to_string("trackmap.toml").unwrap_or(String::new()))
            .expect("invalid trackmap.toml");

    let mut spotify = spotify::Spotify::new(&client_id, &client_secret);

    match spotify.get_user_playlists(&user_id).await {
        Ok(playlists) => {
            let max_length = playlists
                .iter()
                .map(|playlist| playlist.name.len())
                .max()
                .unwrap();

            for i in 0..playlists.len() {
                let total_padding = max_length - playlists[i].name.trim().chars().count() + 2;
                let left_padding = (total_padding as f64 / 2_f64).ceil() as usize;

                println!(
                    "===== {:left$}{}{:right$} =====",
                    "",
                    playlists[i].name.yellow(),
                    "",
                    left = left_padding,
                    right = total_padding - left_padding
                );

                let mut tracks_by_name: HashMap<String, Vec<String>> = HashMap::new();

                match spotify.get_tracks(&playlists[i].id, None).await {
                    Ok(tracks) => {
                        for track in &tracks {
                            let entry = tracks_by_name
                                .entry(track.name.clone())
                                .or_insert_with(Vec::new);

                            if match trackmap.tracknames.get(&track.name) {
                                Some(ids) => !ids.ids.iter().any(|id| entry.contains(id)),
                                None => true,
                            } {
                                entry.push(track.id.clone());
                            }
                        }
                    }
                    Err(e) => throw_error(e),
                }

                let tracks_by_name: HashMap<String, Vec<String>> = tracks_by_name
                    .iter()
                    .filter(|p| p.1.len() > 1)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                for (name, ids) in &tracks_by_name {
                    print!("{}: ", name.cyan());
                    for i in 0..ids.len() {
                        print!(
                            "{}{}",
                            ids[i].bright_red(),
                            if i == ids.len() - 1 { "\n" } else { ", " }
                        );
                    }
                }

                if tracks_by_name.len() == 0 {
                    println!("{}", "no duplicates found".green());
                }

                if i < playlists.len() - 1 {
                    println!();
                }
            }
        }
        Err(e) => throw_error(e),
    }
}
