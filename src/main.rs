use clap::Parser;
use std::process::{Child, Command};
use std::{thread, time::Duration};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    play: String,

    #[arg(short = 'l', long = "loop")]
    loop_song: bool,

    #[arg(short, long)]
    download: bool,
}

fn get_playable_url(mut song: String) -> Result<String, ()> {
    if !song.contains("youtube.com/") && !song.contains("youtu.be/") {
        song = format!("ytsearch1:{}", song);
    }

    let mut cmd = Command::new("yt-dlp");
    cmd.args(["-f", "bestaudio", &song, "--get-url"]);

    let output = cmd.output().map_err(|err| {
        eprintln!("ERROR: command didnt execute {err}");
    })?;

    if !output.status.success() {
        eprintln!("ERROR: youtube command wasnt a success");
        return Err(());
    }

    if let Some(url) = String::from_utf8_lossy(&output.stdout).split("\n").next() {
        return Ok(url.to_string());
    }

    eprintln!("ERROR: couldnt get audio url");
    return Err(());
}

fn play_song(url: &str) -> Result<Child, ()> {
    let mut cmd = Command::new("ffplay");
    cmd.args(["-nodisp", "-autoexit", "-loglevel", "quiet", url]);

    let child = cmd.spawn().map_err(|err| {
        eprintln!("ERROR: Couldnt start song {err}");
    })?;
    return Ok(child);
}

fn main() -> Result<(), ()> {
    let cli = Cli::parse();

    println!("searching song");
    let url = get_playable_url(cli.play)?;

    println!("playing song");
    let mut song = play_song(&url)?;
    song.wait().map_err(|err| {
        eprintln!("ERROR: Couldnt wait for song {err}");
    })?;

    thread::sleep(Duration::from_secs(2));
    while cli.loop_song {
        let mut song = play_song(&url)?;
        song.wait().map_err(|err| {
            eprintln!("ERROR: Couldnt wait for song {err}");
        })?;
        thread::sleep(Duration::from_secs(2));
    }

    return Ok(());
}
