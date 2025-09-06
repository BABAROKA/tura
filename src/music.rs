use rodio::{self, Decoder};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct SongList {
    songs: Vec<Song>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    pub title: String,
    pub duration: u32,
    pub searches: Vec<String>,
}
impl Song {
    pub fn new() -> Self {
        Song {
            title: String::new(),
            duration: 0,
            searches: Vec::new(),
        }
    }
    pub fn play(&self) -> Result<(), ()> {
        let song = format!("{}.m4a", self.title);
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        let file = File::open(song).unwrap();
        let source = Decoder::try_from(file).unwrap();
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }
    pub fn add_song(&self) -> Result<(), ()> {
        let mut song_list = SongList { songs: Vec::new() };
        let file = File::open("./songs/index.json").map_err(|err| {
            eprintln!("Unable to open file to read mode\nError: {err}");
        })?;
        let mut reader = BufReader::new(file);
        let mut read_song_list = String::new();
        reader.read_to_string(&mut read_song_list).map_err(|err| {
            eprintln!("Unable to read file contents to string\nError: {err}");
        })?;

        if let Ok(list) = serde_json::from_str(&read_song_list) {
            song_list = list;
        }

        let mut song_found = false;
        for song in &mut song_list.songs {
            if self.title == song.title {
                song_found = true;
                if song.searches.len() > 3 {
                    break;
                }
                let search = self.searches[0].clone();
                song.searches.push(search);
            }
        }
        if !song_found {
            song_list.songs.push(self.clone());
        }

        let songs_json = serde_json::to_string(&song_list).map_err(|err| {
            eprintln!("Unable to turn string to json\nError: {err}");
        })?;
        let file = File::create("./songs/index.json").map_err(|err| {
            eprintln!("Unable to create file\nError: {err}");
        })?;
        let mut writer = BufWriter::new(file);
        writer.write_all(songs_json.as_bytes()).map_err(|err| {
            eprintln!("Unable to write string to buffer\nError: {err}");
        })?;

        writer.flush().map_err(|err| {
            eprintln!("Unable to write buffer contents to file\nError: {err}");
        })?;

        Ok(())
    }
}

fn get_song(title: &str) -> Option<Song> {
    let file = File::open("./songs/index.json")
        .map_err(|err| {
            eprintln!("Unable to open file to read mode\nError: {err}");
        })
        .ok()?;
    let mut reader = BufReader::new(file);
    let mut read_song_list = String::new();
    reader
        .read_to_string(&mut read_song_list)
        .map_err(|err| {
            eprintln!("Unable to read file contents to string\nError: {err}");
        })
        .ok()?;

    let song_list: SongList = serde_json::from_str(&read_song_list)
        .map_err(|err| {
            eprintln!("Unable to turn string to json\nError: {err}");
        })
        .ok()?;

    for song in song_list.songs {
        if song.title == title {
            return Some(song);
        }
    }
    return None;
}

pub fn download(search: String) -> Result<Song, ()> {
    let path = "songs";

    let mut song = search.clone();
    if !song.contains("youtube.com/") && !song.contains("youtu.be/") {
        song = format!("ytsearch1:{}", song);
    }

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
        "--print",
        "after_move:duration",
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
    let song_duration: u32 = lines
        .next()
        .expect("Unable to get duration")
        .parse()
        .map_err(|err| {
            eprintln!("Unable to parse to u32\nError: {err}");
        })?;

    if let Some(song) = get_song(&song_title) {
        return Ok(song);
    }

    let song = Song {
        title: song_title,
        duration: song_duration,
        searches: vec![search],
    };

    song.add_song();

    Ok(song)
}
