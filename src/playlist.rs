use anyhow::{anyhow, Result};
use clap::Subcommand;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::fs;
use std::path;
use std::process;
use url::Url;

type PlaylistMap<'a> = HashMap<&'a str, &'a str>;

pub(crate) const PLAYLIST_FILE: &str = "playlists.json";

pub(crate) static HAENG_PATH: OnceCell<String> = OnceCell::new();
pub(crate) static FILE_PATH: OnceCell<String> = OnceCell::new();

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Add a YouTube-playlist
    #[command(arg_required_else_help = true)]
    Add {
        /// The name of the playlist
        name: String,

        /// The URL to the playlist
        url: String,
    },
    /// Remove a YouTube-playlist
    #[command(arg_required_else_help = true)]
    Remove {
        /// The name of the playlist that should be removed
        name: String,
    },
    /// Download a specific playlist
    #[command(arg_required_else_help = true)]
    Download {
        /// Save the playlist
        #[arg(short, long)]
        save: bool,

        /// The name of the playlist
        name: String,

        /// The URL to the playlist
        url: Option<String>,
    },
    /// View all playlists currently tracked
    View,
}

pub(crate) fn create_playlist_file() -> Result<()> {
    let file_path = FILE_PATH.get().ok_or(anyhow!("Couldn't get file path"))?;

    if !path::Path::new(file_path).exists() {
        fs::File::create(file_path)?;
    }

    Ok(())
}

pub(crate) fn load_playlists<'a>(data: &'a str) -> Result<PlaylistMap<'a>> {
    match serde_json::from_str(&data) {
        Ok(playlist_map) => Ok(playlist_map),
        Err(_) => Ok(HashMap::new()), // XXX: if the playlists.json file cannot be parsed we override it
    }
}

pub(crate) fn add_playlist<'a>(
    playlists: &mut PlaylistMap<'a>,
    name: &'a str,
    url: &'a str,
) -> Result<()> {
    match Url::parse(url) {
        // Provided value is not a valid URL.
        Err(_) => return Err(anyhow!("`{url}` is not a valid URL")),

        // Provided value is valid, check the domain
        Ok(url) => match url.domain() {
            None => return Err(anyhow!("`{url}` is an invalid URL, expected a domain name")),
            Some(domain) if !matches!(domain, "youtube.com" | "www.youtube.com") => {
                return Err(anyhow!("`{url}` is not a \"youtube.com\" URL"));
            }
            Some(_) => {}
        },
    };

    match playlists.get(name) {
        Some(_) => return Err(anyhow!("Playlist `{name}` already exists")),
        None => playlists.insert(name, url),
    };

    let file_path = FILE_PATH.get().ok_or(anyhow!("Couldn't get file path"))?;
    let file = fs::File::create(file_path)?;
    serde_json::to_writer_pretty(file, &playlists)?;

    println!("Added `{name}` ({url})");

    Ok(())
}

pub(crate) fn remove_playlist(playlists: &mut PlaylistMap<'_>, name: &str) -> Result<()> {
    match playlists.remove(name) {
        Some(url) => println!("Succesfully removed `{name}` ({url})"),
        None => println!("`{name}` was not present in the list"),
    }

    let file_path = FILE_PATH.get().ok_or(anyhow!("Couldn't get file path"))?;
    let file = fs::File::create(file_path)?;
    serde_json::to_writer_pretty(file, &playlists)?;

    Ok(())
}

pub(crate) fn view_playlists(playlists: &PlaylistMap<'_>) -> Result<()> {
    println!("Playlists:");

    playlists.iter().for_each(|(name, url)| {
        println!("\t{name} - {url}");
    });

    Ok(())
}

fn get_url_from_playlists<'a>(playlists: &PlaylistMap<'a>, name: &'a str) -> Result<&'a str> {
    match playlists.get(name) {
        Some(url) => Ok(url),
        None => Err(anyhow!("Playlist `{name}` not found")),
    }
}

fn process_download(name: &str, url: &str) -> Result<()> {
    let haeng_path = HAENG_PATH.get().ok_or(anyhow!("Couldn't get file path"))?;

    process::Command::new("yt-dlp.exe")
        .stdout(process::Stdio::inherit())
        .args([
            "-ciw",
            "-f m4a",
            "--embed-thumbnail",
            "--download-archive",
            &format!("{haeng_path}\\myarchive.txt"),
            "--restrict-filenames",
            "-o",
            &format!("{haeng_path}\\{name}\\%(title)s-%(id)s.%(ext)s"),
            url,
        ])
        .output()
        .map_err(|_| anyhow!("There was an error running YT-DLP"))?;

    Ok(())
}

pub(crate) fn download_playlist<'a>(
    playlists: &mut PlaylistMap<'a>,
    save: bool,
    name: &'a str,
    url: Option<&'a str>,
) -> Result<()> {
    check_for_updates()?;

    println!("{name} - Downloading ...");

    let url = match url {
        Some(url) => url,
        None => get_url_from_playlists(playlists, name)?,
    };

    if save {
        if let Err(_) = add_playlist(playlists, name, url) {
            eprintln!("Playlist already exists, continuing with the download...")
        }
    }

    process_download(name, url)?;

    Ok(())
}

pub(crate) fn download_playlists(playlists: &PlaylistMap<'_>) -> Result<()> {
    check_for_updates()?;

    println!("Starting downloads...");
    playlists
        .iter()
        .map(|(name, url)| process_download(name, url))
        .collect::<Result<()>>()?;

    Ok(())
}

fn check_for_updates() -> Result<()> {
    println!("Checking for updates...");
    process::Command::new("yt-dlp.exe")
        .stdout(process::Stdio::inherit())
        .args(["-U"])
        .output()
        .map_err(|_| anyhow!("There was an error while updating YT-DLP"))?;

    Ok(())
}
