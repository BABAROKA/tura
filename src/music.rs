use std::fs;
use std::process::Command;

#[derive(Debug)]
pub struct Music {
    pub title: String,
    pub duration: i32,
}

impl Music {
    pub fn new(mut song: String, path: &str) -> Result<Music, ()> {
        if !song.contains("youtube.com/") && !song.contains("youtu.be/") {
            song = format!("ytsearch1:{}", song);
        }

        fs::create_dir_all(path).expect("Failed to create music folder");
        let song = format!("ytsearch1:{song}");
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
        let song_duration: i32 = lines
            .next()
            .expect("Unable to get duration")
            .parse()
            .expect("Unable to parse to f32");

        println!("{song_title} - {song_duration}");
        Ok(Music {
            title: song_title,
            duration: song_duration,
        })
    }
}
