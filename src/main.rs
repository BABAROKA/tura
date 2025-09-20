mod music;
use music::SongError;

use clap::Parser;
use crossterm::event::{Event, KeyCode, poll, read};
use ratatui::{DefaultTerminal, Frame};
use std::{thread, time::Duration};

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

fn check_cli(cli: Cli) -> Result<(), SongError> {
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

fn main() {
    let cli = Cli::parse();
    thread::scope(|s| {
        s.spawn(|| {
            check_cli(cli).unwrap();
        });

        s.spawn(|| {
            tui();
        });
    });
}

fn tui() {
    color_eyre::install().unwrap();
    let terminal = ratatui::init();
    run(terminal);
    ratatui::restore();
}

fn run(mut terminal: DefaultTerminal) {
    loop {
        terminal.draw(render).unwrap();
        if poll(Duration::ZERO).unwrap() {
            match read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}
