mod db;
mod music;

use clap::Parser;
use db::Database;
use std::process::{Child, Command};

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

    let mut cmd = Command::new("yt-dlp");
    cmd.args([
        cli.play.as_str(),
        "--skip-download",
        "--get-title",
        "--get-duration",
    ]);
    let output = cmd.output().map_err(|err| {
        eprintln!("Unable to get output from yt-dlp command {err}")
    })?;
    if !output.status.success() {
        eprintln!("Output wasnt a success");
        return Err(());
    }
    let output_sting = String::from_utf8(output.stdout).map_err(|err| {
        eprintln!("Unable to convert output to string {err}");
    })?;
    let data: Vec<&str> = output_sting.split("\n").collect();
    println!("{:?}", data);

    return Ok(());
}
