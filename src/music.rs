use rodio::{self, Decoder, decoder::DecoderError, stream::StreamError};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::process::Command;
use strsim::normalized_levenshtein as lstein;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SongError {
    #[error("Failed to perform I/O operation: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to work with json data: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Failed to parse string to other type: {0}")]
    Parse(#[from] ParseIntError),

    #[error("Failed to stream music: {0}")]
    Stream(#[from] StreamError),

    #[error("Failed to decode music data: {0}")]
    Decode(#[from] DecoderError),

    #[error("yt-dlp command failed: {0}")]
    YtDlpError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongList {
    songs: Vec<Song>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    id: String,
    title: String,
    duration: u32,
    searches: Vec<String>,
}
impl Song {
    fn play(&self) -> Result<(), SongError> {
        let song = format!("{0}/{1}.m4a", songs_dir().to_str().unwrap(), self.id);
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        let file = File::open(song)?;
        let source = Decoder::try_from(file)?;
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }

    fn add_song(&self) -> Result<(), SongError> {
        let mut song_list = SongList { songs: Vec::new() };
        let path = format!("{}/index.json", songs_dir().to_str().unwrap());
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut read_song_list = String::new();
        reader.read_to_string(&mut read_song_list)?;

        if let Ok(list) = serde_json::from_str(&read_song_list) {
            song_list = list;
        }

        let mut song_found = false;
        for song in &mut song_list.songs {
            if self.id == song.id {
                song_found = true;
            }
        }

        if !song_found {
            song_list.songs.push(self.clone());
        }

        let songs_json = serde_json::to_string(&song_list)?;
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(songs_json.as_bytes())?;

        writer.flush()?;

        Ok(())
    }

    fn add_search(&self, search: String) -> Result<(), SongError> {
        let path = format!("{}/index.json", songs_dir().to_str().unwrap());
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut read_song_list = String::new();
        reader.read_to_string(&mut read_song_list)?;

        let mut song_list: SongList = serde_json::from_str(&read_song_list)?;

        for song in &mut song_list.songs {
            if self.id == song.id {
                if song.searches.len() == 3 {
                    song.searches.remove(0);
                }
                song.searches.push(search.clone());
            }
        }

        let songs_json = serde_json::to_string(&song_list)?;
        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(songs_json.as_bytes())?;

        writer.flush()?;

        Ok(())
    }
}

pub fn get_all_songs() -> Option<SongList> {
    let path = format!("{}/index.json", songs_dir().to_str().unwrap());
    let file = File::open(&path).ok()?;
    let mut reader = BufReader::new(file);
    let mut read_song_list = String::new();
    reader.read_to_string(&mut read_song_list).ok()?;

    let song_list: SongList = serde_json::from_str(&read_song_list).ok()?;
    if song_list.songs.len() > 0 {
        return Some(song_list);
    }
    return None;
}

pub fn play_song(title: &str) {
    if let Some(song) = get_best_match(title) {
        song.add_search(title.to_string()).unwrap();
        song.play().unwrap();
    } else {
        println!("Song not found in index\nDownloading");
        let song = download(title.to_owned());
        match song {
            Ok(song) => {
                song.add_search(title.to_string()).unwrap();
                song.play().unwrap();
            }
            Err(_) => return,
        }
    }
}

fn download(search: String) -> Result<Song, SongError> {
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
        songs_dir().to_str().unwrap(),
        "-o",
        "%(id)s.%(ext)s",
        "--print",
        "after_move:title",
        "--print",
        "after_move:duration",
        "--print",
        "after_move:id",
        &song,
    ]);

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SongError::YtDlpError(stderr.to_string()));
    }
    let output_sting = String::from_utf8_lossy(&output.stdout);
    let mut lines = output_sting.lines();
    let song_title = lines.next().expect("Unable to get title").to_string();
    let song_duration: u32 = lines.next().expect("Unable to get duration").parse()?;
    let song_id = lines.next().expect("Unable to get id").to_string();

    if let Some(list) = get_all_songs() {
        for song in list.songs {
            if song.id == song_id {
                return Ok(song);
            }
        }
    }

    let song = Song {
        id: song_id,
        title: song_title,
        duration: song_duration,
        searches: vec![search],
    };

    song.add_song()?;
    Ok(song)
}

fn get_best_match(title: &str) -> Option<Song> {
    let song_list = get_all_songs()?;

    let song = song_list
        .songs
        .into_iter()
        .map(|song| {
            let mut search_difference: f64 = 0_f64;
            for search in &song.searches {
                search_difference += lstein(search, title);
            }
            search_difference /= song.searches.len() as f64;

            let difference = (lstein(title, &song.title) * 0.1) + (search_difference * 0.9);
            return (song, difference);
        })
        .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal))?;

    println!("{}", song.1);
    if song.1 < 0.75 {
        return None;
    }
    return Some(song.0);
}

fn songs_dir() -> PathBuf {
    let mut exe_dir = std::env::current_exe().expect("Failed to get exe path");
    exe_dir.pop();
    exe_dir.push("songs");
    exe_dir
}
