use rodio::{self, Decoder, decoder::DecoderError, stream::StreamError};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::mem::swap;
use std::num::ParseIntError;
use std::process::Command;
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
        let song = format!("./songs/{}.m4a", self.id);
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
        let file = File::open("./songs/index.json")?;
        let mut reader = BufReader::new(file);
        let mut read_song_list = String::new();
        reader.read_to_string(&mut read_song_list)?;

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

        let songs_json = serde_json::to_string(&song_list)?;
        let file = File::create("./songs/index.json")?;
        let mut writer = BufWriter::new(file);
        writer.write_all(songs_json.as_bytes())?;

        writer.flush()?;

        Ok(())
    }
}

fn play_song(title: &str) {
    if let Some(song) = get_best_match(title) {
        song.play();
    }
}

fn get_song(title: &str) -> Option<Song> {
    let file = File::open("./songs/index.json").ok()?;
    let mut reader = BufReader::new(file);
    let mut read_song_list = String::new();
    reader.read_to_string(&mut read_song_list).ok()?;

    let song_list: SongList = serde_json::from_str(&read_song_list).ok()?;

    for song in song_list.songs {
        if song.title == title {
            return Some(song);
        }
    }
    return None;
}

pub fn download(search: String) -> Result<Song, SongError> {
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

    if let Some(song) = get_song(&song_title) {
        return Ok(song);
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

pub fn get_all_songs() -> Option<SongList> {
    let file = File::open("./songs/index.json").ok()?;
    let mut reader = BufReader::new(file);
    let mut read_song_list = String::new();
    reader.read_to_string(&mut read_song_list).ok()?;

    let song_list: SongList = serde_json::from_str(&read_song_list).ok()?;
    if song_list.songs.len() > 0 {
        return Some(song_list);
    }
    return None;
}

fn get_best_match(title: &str) -> Option<Song> {
    let song_list = get_all_songs()?;

    let song = song_list
        .songs
        .into_iter()
        .map(|song| {
            let difference = lstein(title, &song.title);
            return (song, difference);
        })
        .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal))?;

    return Some(song.0);
}

fn lstein(first: &str, second: &str) -> f32 {
    let m = first.len();
    let n = second.len();

    if m == 0 && n == 0 {
        return 1.0;
    }

    let first_chars: Vec<char> = first.to_lowercase().chars().collect();
    let second_chars: Vec<char> = second.to_lowercase().chars().collect();

    let mut v0: Vec<usize> = (0..=n).collect();
    let mut v1: Vec<usize> = vec![0; n + 1];

    for i in 0..m {
        v1[0] = i + 1;

        for j in 0..n {
            let del_cost = v0[j + 1] + 1;
            let inser_cost = v1[j] + 1;
            let sub_cost = if first_chars[i] == second_chars[j] {
                v0[j]
            } else {
                v0[j] + 1
            };
            v1[j + 1] = del_cost.min(inser_cost).min(sub_cost);
        }
        swap(&mut v0, &mut v1);
    }
    let distance = v0[n];
    let max_len = m.max(n);

    1.0 - (distance as f32 / max_len as f32)
}
