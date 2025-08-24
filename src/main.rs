use clap::Parser;
use std::process::Command;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    play: String,

    #[arg(short = 'l', long = "loop")]
    loop_song: bool,
}

fn get_playable_url(mut song: String) -> Result<String, ()> {
    if !song.contains("youtube.com/") || !song.contains("youtu.be/") {
        song = format!("ytsearch1:{}", song);
    }

    let mut cmd = Command::new("yt-dlp");
    cmd.args(["-f", "bestaudio", &song, "--get-url"]);

    let output = cmd.output().map_err(|err| {
        eprintln!("ERROR: command didnt execute");
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
    return Ok(());
}
