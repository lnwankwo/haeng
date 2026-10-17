use anyhow::{anyhow, Result};
use clap::Subcommand;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process;
use url::Url;

type PlaylistMap<'a> = HashMap<&'a str, &'a str>;

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Add a YouTube playlist
    #[command(arg_required_else_help = true)]
    Add {
        /// The name of the playlist
        name: String,

        /// The URL to the playlist
        url: String,
    },
    /// Remove a YouTube playlist
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

pub(crate) fn ensure_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        fs::File::create(path)?;
    }

    Ok(())
}

pub(crate) fn load_playlists<'a>(data: &'a str) -> Result<PlaylistMap<'a>> {
    match serde_json::from_str(&data) {
        Ok(playlist_map) => Ok(playlist_map),
        Err(_) => Ok(HashMap::new()), // XXX: if the playlists.json file cannot be parsed we override it
    }
}

/// This overwrites the config by truncating it and writing the new playlists.
pub(crate) fn save_playlists(path: &PathBuf, playlists: &PlaylistMap) -> Result<()> {
    // This overwrites the config by truncating it and writing the new playlists.
    let file = fs::File::create(path)?;
    serde_json::to_writer_pretty(file, &playlists)?;

    Ok(())
}

pub(crate) fn add_playlist<'a>(
    path: &'a PathBuf,
    playlists: &mut PlaylistMap<'a>,
    name: &'a str,
    url: &'a str,
) -> Result<()> {
    validate_url(url)?;

    match playlists.get(name) {
        Some(_) => return Err(anyhow!("Playlist `{name}` already exists")),
        None => playlists.insert(name, url),
    };

    save_playlists(path, playlists)?;

    println!("Added `{name}` ({url})");

    Ok(())
}

fn validate_url(url: &str) -> Result<()> {
    match Url::parse(url) {
        // Provided value is not a valid URL.
        Err(_) => Err(anyhow!("`{url}` is not a valid URL")),

        // Provided value is valid, check the domain
        Ok(url) => match url.domain() {
            None => Err(anyhow!("`{url}` is an invalid URL, expected a domain name")),
            Some(domain) if !matches!(domain, "youtube.com" | "www.youtube.com") => {
                Err(anyhow!("`{url}` is not a \"youtube.com\" URL"))
            }
            Some(_) => Ok(()),
        },
    }
}

pub(crate) fn remove_playlist(
    path: &PathBuf,
    playlists: &mut PlaylistMap<'_>,
    name: &str,
) -> Result<()> {
    match playlists.remove(name) {
        Some(url) => println!("Succesfully removed `{name}` ({url})"),
        None => println!("`{name}` was not present in the list"),
    }

    save_playlists(path, playlists)?;

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

fn process_download(path: &PathBuf, name: &str, url: &str) -> Result<()> {
    process::Command::new("yt-dlp.exe")
        .stdout(process::Stdio::inherit())
        .args([
            "-ciw",
            "-f m4a",
            "--embed-thumbnail",
            "--download-archive",
            path.join("myarchive.txt")
                .to_str()
                .expect("Path not valid unicode"),
            "--restrict-filenames",
            "-o",
            path.join(name)
                .join("%(title)s-%(id)s.%(ext)s")
                .to_str()
                .expect("Path not valid unicode"),
            url,
        ])
        .output()
        .map_err(|_| anyhow!("There was an error running YT-DLP"))?;

    Ok(())
}

pub(crate) fn download_playlist<'a>(
    path: &'a PathBuf,
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

    validate_url(url)?;

    if save {
        if let Err(_) = add_playlist(path, playlists, name, url) {
            eprintln!("Playlist already exists, continuing with the download...")
        }
    }

    process_download(path, name, url)?;

    Ok(())
}

pub(crate) fn download_playlists(path: &PathBuf, playlists: &PlaylistMap<'_>) -> Result<()> {
    check_for_updates()?;

    println!("Starting downloads...");
    playlists
        .iter()
        .map(|(name, url)| process_download(path, name, url))
        .collect::<Result<()>>()?;

    Ok(())
}

fn check_for_updates() -> Result<()> {
    println!("Checking for updates...");
    process::Command::new("yt-dlp")
        .stdout(process::Stdio::inherit())
        .args(["-U"])
        .output()
        .map_err(|_| anyhow!("There was an error while updating YT-DLP"))?;

    Ok(())
}
