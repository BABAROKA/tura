use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::mem::swap;

pub fn get_all_songs() -> Option<Vec<String>> {
    let mut songs: Vec<String> = Vec::new();

    let file = match File::open("./songs/index.txt") {
        Ok(f) => f,
        Err(_) => File::create("./songs/index.txt").expect("Unable to create file"),
    };
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
    let file = OpenOptions::new()
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
