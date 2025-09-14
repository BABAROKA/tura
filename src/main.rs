mod music;
use music::SongError;

use clap::Parser;

#[derive(Parser)]
#[command(version,name="Tura",about="CLI Oflline Music Player", long_about = None)]
struct Cli {
    song: Option<String>,

    #[arg(short, long)]
    download: bool,

    #[arg(short = 'l', long = "loop")]
    loop_song: bool,

    #[arg(short, long)]
    remove: bool,

    #[arg(short, long)]
    all_songs: bool,
}

fn main() -> Result<(), SongError> {
    let cli = Cli::parse();

    if cli.all_songs {
        music::show_songs()?;
    }

    if cli.remove {
        match cli.song {
            Some(song) => music::remove_song(&song)?,
            None => println!("No song was written"),
        }
        return Ok(());
    }

    if cli.loop_song {
        match cli.song {
            Some(song) => music::loop_song(&song, cli.download)?,
            None => println!("No song was written"),
        }
        return Ok(());
    }
    if let Some(song) = cli.song {
        music::play_song(&song, cli.download)?;
    }
    Ok(())
}
