mod music;
mod index;

use clap::Parser;
use std::mem::swap;

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


fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    let songs = index::get_all_songs();
    println!("{:?}", songs);

    return Ok(());
}
