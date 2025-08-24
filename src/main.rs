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

fn main() {
    let cli = Cli::parse();
    let mut cmd = Command::new("yt-dlp");
    let url: String;
    cmd.args(["-f", "bestaudio", "ytsearch1:the fat rat monody", "--get-url"]);

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(url_str) = stdout.split("\n").next() {
                    url = url_str.to_string();
                    println!("playing this song {song}", song = url);
                } else {
                    panic!("No song found");
                }
                return;
            }
        }
        Err(err) => {
            panic!("Couldnt get song {err}");
        }
    }
}
