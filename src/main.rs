use clap::Parser;
use std::fs;

mod playlist;
use playlist::*;

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
    HAENG_PATH
        .set(std::env::var("HAENG_PATH").expect("Failed to load `HAENG_PATH`"))
        .expect("Failed to get `HAENG_PATH`");
    FILE_PATH
        .set(format!("{}\\{PLAYLIST_FILE}", HAENG_PATH.get().unwrap()))
        .expect("Failed to set playlists file path");

    let file_path = FILE_PATH.get().expect("Failed to get file path");

    create_playlist_file().expect("Failed to create playlists file");

    let args = Cli::parse();
    let data = fs::read_to_string(&file_path).expect("Failed to read `playlists.json`");
    let mut playlists = load_playlists(&data).expect("Failed to load playlists");

    let result = match args.command {
        Some(Commands::Add { name, url }) => add_playlist(&mut playlists, &name, &url),
        Some(Commands::Remove { name }) => remove_playlist(&mut playlists, &name),
        Some(Commands::View) => view_playlists(&playlists),
        Some(Commands::Download { save, name, url }) => {
            download_playlist(&mut playlists, save, &name, url.as_deref())
        }
        None => download_playlists(&mut playlists),
    };

    if let Err(e) = result {
        eprintln!("ERROR: {e}");
        std::process::exit(1);
    }
}
