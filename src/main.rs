mod index;
mod music;

use std::fmt::format;

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

fn main() -> Result<(), ()> {
    let cli = Cli::parse();

    if cli.download {
        music::download(cli.play)?;
        return Ok(());
    }

    if let Some(song) = index::get_best_match(&cli.play) {
        music::play(format!("./songs/{song}.m4a"))?;
    }
    return Ok(());
}
