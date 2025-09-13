mod music;
use music::SongError;

use clap::Parser;

#[derive(Parser)]
#[command(version,name="Tura",about="CLI Oflline Music Player", long_about = None)]
struct Cli {
    song: String,

    #[arg(short, long)]
    download: bool,

    #[arg(short = 'l', long = "loop")]
    loop_song: bool,
}

fn main() -> Result<(), SongError> {
    let cli = Cli::parse();

    if cli.loop_song {
        music::loop_song(&cli.song, cli.download)?;
    }
    music::play_song(&cli.song, cli.download)?;
    Ok(())
}
