mod music;
use music::SongError;

mod tui;
use tui::Action;

use clap::Parser;
use std::sync::mpsc::{Sender, channel};
use std::thread;

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

fn check_cli(cli: Cli, tx: Sender<Action>) -> Result<(), SongError> {
    if cli.all_songs {
        music::show_songs()?;
    }

    if cli.remove {
        match cli.song {
            Some(song) => music::remove_song(song)?,
            None => println!("No song was written"),
        }
        return Ok(());
    }

    if cli.loop_song {
        match cli.song {
            Some(song) => music::loop_song(song, &tx)?,
            None => println!("No song was written"),
        }
        return Ok(());
    }
    if let Some(song) = cli.song {
        music::play_song(song, cli.download, &tx)?;
    }
    tx.send(Action::Quit).unwrap();
    Ok(())
}

fn main() {
    let (tx, rx) = channel();
    let cli = Cli::parse();
    let song = cli.song.clone();

    thread::spawn(|| check_cli(cli, tx));
    if let Some(_) = song {
        tui::init(rx);
    } else {
        rx.recv().unwrap();
    }
}
