mod music;

use clap::Parser;
use music::Song;

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

fn main() {
    let song = Song::new();
    println!("{song:?}");
}
