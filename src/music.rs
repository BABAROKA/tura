use rodio::{self, Decoder};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::mem::swap;
use std::process::Command;

#[derive(Debug)]
pub struct Music {
    title: String,
    duration: usize,
    searches: Vec<String>,
}
impl Music {
    pub fn open_index(io: char) -> Result<File, ()> {
        fs::create_dir_all("songs").map_err(|err| {
            eprintln!("Unable to create songs folder\nError: {err}");
        })?;
        if io == 'r' {
            let file = File::options()
                .create(true)
                .open("/songs/index.json")
                .map_err(|err| {
                    eprintln!("Unable to open file in read mode\nError: {err}");
                })?;
            return Ok(file);
        }
        if io == 'w' {
            let file = File::options()
                .create(true)
                .append(true)
                .open("/songs/index.json")
                .map_err(|err| {
                    eprintln!("Unable to open file in append mode\nError: {err}");
                })?;
            return Ok(file);
        }

        return Err(());
    }
    pub fn get_all_songs() -> Option<Vec<String>> {
        let mut songs: Vec<String> = Vec::new();
        let file = open_index('r').ok()?;

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap_or("".to_string());
            songs.push(line.trim().to_string());
        }
        if songs.len() > 0 {
            return Some(songs);
        }
        return None;
    }
}

pub fn get_song(title: &str) -> Option<String> {
    let file = match File::open("./songs/index.txt") {
        Ok(f) => f,
        Err(_) => File::create("./songs/index.txt").expect("Unable to create file"),
    };
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap_or("".to_string());
        if title.trim() == line.trim() {
            return Some(title.to_string());
        }
    }
    return None;
}

pub fn write_song(title: &str) -> Result<(), ()> {
    let file = File::options()
        .create(false)
        .append(true)
        .open("./songs/index.txt")
        .map_err(|err| {
            eprintln!("Unable to open writeable file {err}");
        })?;

    let mut writer = BufWriter::new(file);
    writeln!(writer, "{title}").map_err(|err| {
        eprintln!("Unable to write to file {err}");
    })?;
    return Ok(());
}

pub fn get_best_match(title: &str) -> Option<String> {
    if let Some(songs) = get_all_songs() {
        let mut best: (String, f32) = (String::new(), 0.0);
        for song in songs {
            let difference = lstein(&song, title);
            if best.1 < difference {
                best = (song, difference);
            }
        }
        return Some(best.0);
    }
    return None;
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
pub fn download(mut song: String) -> Result<String, ()> {
    let path = "songs";
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

    if let Some(title) = get_song(&song_title) {
        return Ok(title);
    }

    write_song(&song_title)?;

    Ok(song_title)
}

pub fn play(song: String) -> Result<(), ()> {
    let stream_handle =
        rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
    let sink = rodio::Sink::connect_new(&stream_handle.mixer());
    let file = File::open(song).unwrap();
    let source = Decoder::try_from(file).unwrap();
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}
