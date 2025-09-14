use rodio::Source;
use rodio::{self, Decoder, decoder::DecoderError, stream::StreamError};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
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
        write_all_songs(&song_list)?;
        Ok(())
    }

    fn add_search(new_song: &Song, search: &str) -> Result<(), SongError> {
        let mut song_list: SongList = get_all_songs()?;

        if let Some(song) = song_list.songs.iter_mut().find(|s| s.id == new_song.id) {
            let score = song
                .searches
                .last()
                .map(|last| lstein(&last, search))
                .unwrap_or(0.0);
            if score < 0.7 {
                return Ok(());
            }
            if song.searches.len() == 3 {
                song.searches.rotate_left(1);
                song.searches.pop();
            }
            song.searches.push(search.to_owned());
        }

        return write_all_songs(&song_list);
    }
    fn remove(old_song: &Song) -> Result<(), SongError> {
        let mut song_list: SongList = get_all_songs()?;
        song_list.songs.retain(|song| song.id != old_song.id);
        write_all_songs(&song_list)?;
        Ok(())
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
    let song_count = 1;
    if !download {
        if let Some(songs) = get_best_match(title, song_count) {
            let (song, score) = songs.first().unwrap();
            println!("Found best score {}: '{}'", *score as f32, song.title);
            if *score < 0.75 {
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
    let song_count = 1;
    if let Some(songs) = get_best_match(title, song_count) {
        let (song, score) = songs.first().unwrap();
        println!("Found best score {}: '{}'", *score as f32, song.title);
        if download {
            println!("Unable to download when looping");
        }
        SongList::add_search(&song, title)?;
        song.play(do_loop)?;
    }
    Ok(())
}

pub fn remove_song(title: &str) -> Result<(), SongError> {
    let song_count = 3;
    if let Some(songs) = get_best_match(title, song_count) {
        println!("Found three best matches. Which one to remove");
        for (i, (song, score)) in songs.iter().enumerate() {
            println!(
                " - {i}. {song} - {score}",
                i = i + 1,
                song = song.title,
                score = *score as f32
            );
        }
        print!("\nEnter the number of the song you want to remove: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let song_num: usize = input.trim().parse().unwrap();
        if song_num > 3 || song_num < 1 {
            println!("Not a number in the list");
            return Ok(());
        }
        let song = &songs[song_num - 1].0;
        SongList::remove(song)?;
    }

    Ok(())
}

pub fn show_songs() -> Result<(), SongError> {
    let song_list = get_all_songs()?;
    println!("\n{} Songs", song_list.songs.len());
    for song in song_list.songs {
        println!(" - {}", song.title);
    }
    Ok(())
}

fn download_song(search: String) -> Result<Song, SongError> {
    let song: String = if !search.contains("youtube.com/") && !search.contains("youtu.be/") {
        format!("ytsearch1:{}", search)
    } else {
        search
    };

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
        searches: vec![song],
    };

    SongList::add_song(&song)?;
    Ok(song)
}

fn get_all_songs() -> Result<SongList, SongError> {
    let path = songs_dir().join("index.json");
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let song_list: SongList = serde_json::from_reader(reader)?;
    Ok(song_list)
}

fn write_all_songs(song_list: &SongList) -> Result<(), SongError> {
    let path = songs_dir().join("index.json");
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, song_list)?;
    writer.flush()?;
    Ok(())
}

fn get_best_match(title: &str, song_count: usize) -> Option<Vec<(Song, f64)>> {
    let song_list = get_all_songs().ok()?;
    if song_list.songs.is_empty() {
        return None;
    }
    if song_count == 0 {
        return None;
    }

    let mut data: Vec<(Song, f64)> = song_list
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
        .collect();

    data.sort_by(|a, b| b.1.total_cmp(&a.1));
    let songs: Vec<(Song, f64)> = data.into_iter().take(song_count).collect();
    Some(songs)
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
