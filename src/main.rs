use clap::Parser;
use std::fs;

mod playlist;

use crate::playlist::{
    add_playlist, download_playlist, download_playlists, ensure_exists, load_playlists,
    remove_playlist, view_playlists, Commands,
};

const DESCRIPTION: &str = r#"
If no command is provided, haeng will try to
download all currently saved playlists."#;

#[derive(Debug, Parser)]
#[command(name = "haeng")]
#[command(about = "A playlist download manager.", long_about = DESCRIPTION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() {
    let home = dirs::home_dir().expect("Could not find home directory");
    let work_directory = home.join(".haeng");
    let playlists_file = work_directory.join("playlists.json");

    fs::create_dir_all(work_directory).expect("Could not create .haeng directory");
    ensure_exists(&playlists_file).expect("Failed to create playlists file");

    let data = fs::read_to_string(&playlists_file).expect("Failed to read `playlists.json`");
    let mut playlists = load_playlists(&data).expect("Failed to load playlists");

    let args = Cli::parse();
    let result = match args.command {
        Some(Commands::Add { name, url }) => {
            add_playlist(&playlists_file, &mut playlists, &name, &url)
        }
        Some(Commands::Remove { name }) => remove_playlist(&playlists_file, &mut playlists, &name),
        Some(Commands::View) => view_playlists(&playlists),
        Some(Commands::Download { save, name, url }) => {
            download_playlist(&playlists_file, &mut playlists, save, &name, url.as_deref())
        }
        None => download_playlists(&playlists_file, &playlists),
    };

    if let Err(e) = result {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}
