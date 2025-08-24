use clap::Parser;
use reqwest::blocking::get;
use std::process::{Command};
use rodio::{OutputStreamBuilder, Sink, Decoder};
use std::io::{Cursor, Read};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    play: String,

    #[arg(short = 'l', long = "loop")]
    loop_song: bool,
}

fn get_playable_url(mut song: String) -> Result<String, ()> {
    println!("getting song");
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

fn main() -> Result<(), ()>{
    let cli = Cli::parse();
    let url = get_playable_url(cli.play)?;

    let stream_handle = OutputStreamBuilder::open_default_stream().expect("open default audio stream");
    let sink = Sink::connect_new(&stream_handle.mixer());

    println!("staring song2");
    let mut response = get(&url).map_err(|err| {
        eprintln!("ERROR: Failed to get URL {err}");
    })?;

    let mut buffer: Vec<u8> = Vec::new();
    response.read_to_end(&mut buffer).map_err(|err| {
        eprintln!("ERROR: Reading into buffer {err}");
    })?;

    let cursor = Cursor::new(buffer);
    let source = Decoder::new(cursor).map_err(|err| {
        eprintln!("ERROR: Couldnt decode buffer {err}");
    })?;
    println!("staring song");
    sink.append(source);
    sink.sleep_until_end();
    
    return Ok(());
}
