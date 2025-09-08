mod music;

use clap::Parser;

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
    let cli = Cli::parse();
    if let Some(song) = music::get_best_match(&cli.play) {
        println!("{:?}", song);
        song.play();
    }
}
