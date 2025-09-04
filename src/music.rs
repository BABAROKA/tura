use std::process::Command;
use std::fs::{File, self};
use rodio::{Decoder, OutputStream, source::Source, self};

use crate::index;

pub fn download(mut song: String) -> Result<String, ()> {
    let path = "songs";
    if !song.contains("youtube.com/") && !song.contains("youtu.be/") {
        song = format!("ytsearch1:{}", song);
    }

    fs::create_dir_all(path).expect("Failed to create songs folder");

    let mut cmd = Command::new("yt-dlp");
    cmd.args([
        "-x",
        "--audio-format",
        "m4a",
        "-P",
        path,
        "-o",
        "%(title)s.%(ext)s",
        "--print",
        "after_move:title",
        &song,
    ]);

    let output = cmd
        .output()
        .map_err(|err| eprintln!("Unable to get output from yt-dlp command {err}"))?;
    if !output.status.success() {
        eprintln!("Output wasnt a success {:?}", output);
        return Err(());
    }
    let output_sting = String::from_utf8_lossy(&output.stdout);
    let mut lines = output_sting.lines();
    let song_title = lines.next().expect("Unable to get title").to_string();

    if let Some(title) = index::get_song(&song_title) {
        return Ok(title);
    }

    index::write_song(&song_title)?;

    Ok(song_title)
}

pub fn play(song: String) -> Result<(), ()> {
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");
    let sink = rodio::Sink::connect_new(&stream_handle.mixer());
    let file = File::open(song).unwrap();
    let source = Decoder::try_from(file).unwrap();
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
