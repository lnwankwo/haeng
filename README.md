# Haeng

Haeng is a basic command-line interface (CLI) that I use to manage and download YouTube playlists.

# Dependencies

-   [yt-dlp](https://github.com/yt-dlp/yt-dlp) should be installed and in `PATH`

# Installation

Currently, I have not created any install setup. If you wish to use this, you can build the executable using the following commands:

```
git clone https://github.com/lnwankwo/haeng
cd haeng
cargo build --release
```

Subsequently, you can place the executable in `haeng/target/release` anywhere on your system and add `path/to/executable` to your `PATH` environment variable.

# Usage

`haeng` relies on the `PATH` variable `HAENG_PATH` to find the folder in which playlists are saved, which can be set manually or configured in your system variables. If you wish to set the variable when calling `haeng.exe`, please use:

```
HAENG_PATH="path/to/playlist_folder" haeng.exe [COMMAND]
```

Also see the usage message below:

```
If no command is provided, haeng will try to
download all currently saved playlists.

Usage: haeng.exe [COMMAND]

Commands:
  add
          Add a YouTube-playlist
  remove
          Remove a YouTube-playlist
  download
          Download a specific playlist
  view
          View all playlists currently tracked
  help
          Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')
```

# Notes

-   The tool accepts youtube URLs that are not playlists:

    -   If the provided URL points to a single video, this video will be downloaded.
    -   If the provided URL does not point to a playlist or video, an empty folder will be created when attempting to download the "playlist".

-   If the playlist file cannot be parsed, it is overridden. This might be fixed in the future, but should not happen if you do not manually edit the file.
