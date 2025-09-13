use rodio::Source;
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

    #[error("Yt-dlp command failed: {0}")]
    YtDlpError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SongList {
    songs: Vec<Song>,
}
impl SongList {
    fn add_song(new_song: &Song) -> Result<(), SongError> {
        let mut song_list: SongList = get_all_songs()?;
        if let Some(_) = song_list.songs.iter_mut().find(|s| s.id == new_song.id) {
            return Ok(());
        }
        song_list.songs.push(new_song.clone());
        write_all_songs(song_list)?;
        Ok(())
    }

    fn add_search(new_song: &Song, search: &str) -> Result<(), SongError> {
        let mut song_list: SongList = get_all_songs()?;

        if let Some(song) = song_list.songs.iter_mut().find(|s| s.id == new_song.id) {
            if song.searches.len() == 3 {
                song.searches.remove(0);
            }
            song.searches.push(search.to_owned());
        }

        return write_all_songs(song_list);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    id: String,
    title: String,
    duration: u32,
    searches: Vec<String>,
}
impl Song {
    fn play(&self, loop_song: bool) -> Result<(), SongError> {
        let song = songs_dir().join(format!("{}.m4a", self.id));
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        let file = File::open(song)?;
        if loop_song {
            let source = Decoder::try_from(file)?.repeat_infinite();
            sink.append(source);
            sink.sleep_until_end();
            return Ok(());
        }
        let source = Decoder::try_from(file)?;
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }
}

pub fn play_song(title: &str, download: bool) -> Result<(), SongError> {
    let dont_loop = false;
    if !download {
        if let Some((song, lstein)) = get_best_match(title) {
            println!("Found best score {}: '{}'", lstein as f32, song.title);
            if lstein < 0.75 {
                println!("Use -d to download the correct song");
            }
            SongList::add_search(&song, title)?;
            song.play(dont_loop)?;
            return Ok(());
        }
    }

    println!("Downloading song: '{}'", title);
    let song = download_song(title.to_string())?;

    println!("Found song: '{}'", song.title);
    SongList::add_search(&song, title)?;
    song.play(dont_loop)?;
    Ok(())
}

pub fn loop_song(title: &str, download: bool) -> Result<(), SongError> {
    let do_loop = true;
    if let Some((song, lstein)) = get_best_match(title) {
        println!("Found best score {}: '{}'", lstein as f32, song.title);
        if download {
            println!("Unable to download when looping");
        }
        SongList::add_search(&song, title)?;
        song.play(do_loop)?;
    }
    Ok(())
}

fn download_song(search: String) -> Result<Song, SongError> {
    let mut song = search.clone();
    if !song.contains("youtube.com/") && !song.contains("youtu.be/") {
        song = format!("ytsearch1:{}", song);
    }

    let mut cmd = Command::new("yt-dlp");
    cmd.args([
        "--no-playlist",
        "-x",
        "--audio-format",
        "m4a",
        "-P",
        songs_dir().to_string_lossy().as_ref(),
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
    let song_title = lines
        .next()
        .ok_or(SongError::YtDlpError("Missing Title".to_string()))?
        .to_string();
    let song_duration: u32 = lines
        .next()
        .ok_or(SongError::YtDlpError("Missing Duration".to_string()))?
        .parse()?;
    let song_id = lines
        .next()
        .ok_or(SongError::YtDlpError("Missing Id".to_string()))?
        .to_string();

    let song_list = get_all_songs()?;
    if let Some(song) = song_list.songs.into_iter().find(|s| s.id == song_id) {
        return Ok(song);
    }

    let song = Song {
        id: song_id,
        title: song_title,
        duration: song_duration,
        searches: vec![search],
    };

    SongList::add_song(&song)?;
    Ok(song)
}

fn get_all_songs() -> Result<SongList, SongError> {
    let path = songs_dir().join("index.json");
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut read_song_list = String::new();
    reader.read_to_string(&mut read_song_list)?;

    let song_list: SongList = serde_json::from_str(&read_song_list)?;
    Ok(song_list)
}

fn write_all_songs(song_list: SongList) -> Result<(), SongError> {
    let path = songs_dir().join("index.json");
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &song_list)?;
    writer.flush()?;
    Ok(())
}

fn get_best_match(title: &str) -> Option<(Song, f64)> {
    let song_list = get_all_songs().ok()?;
    if song_list.songs.len() == 0 {
        return None;
    }

    let data = song_list
        .songs
        .into_iter()
        .map(|song| {
            let search_difference: f64 = song
                .searches
                .iter()
                .map(|search| lstein(&search, title))
                .sum::<f64>()
                / song.searches.len() as f64;

            let difference = (lstein(title, &song.title) * 0.1) + (search_difference * 0.9);
            return (song, difference);
        })
        .max_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal))?;

    Some(data)
}

fn songs_dir() -> PathBuf {
    let mut dir = std::env::current_exe().expect("Failed to get exe path");
    dir.pop();
    dir.push("songs");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create songs directory");
    }

    let index_path = dir.join("index.json");
    if !index_path.exists() {
        let empty_list = SongList { songs: vec![] };
        let file = File::create(&index_path).expect("Failed to create index.json");
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &empty_list).expect("Failed to write empty index.json");
        writer.flush().expect("Failed to write to json file");
    }
    dir
}
