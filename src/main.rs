#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use flexi_logger::Logger;
use log::{error, info};

use tokio::task;

slint::slint!(export { ytdlrs } from "ui/main.slint";);

const YT_DLP: &[u8] = include_bytes!("../resources/yt-dlp.exe");
const FFMPEG: &[u8] = include_bytes!("../resources/ffmpeg.exe");

fn write_to_temp_file(data: &[u8], filename: &str) -> io::Result<std::path::PathBuf> {
    let temp_dir = env::temp_dir();
    let temp_path = temp_dir.join(filename);

    let file = File::create(&temp_path);
    if file.is_err() {
        error!("couldn't write {filename}");
    }
    let mut file = file.unwrap();
    if file.write_all(data).is_err() {
        error!("couldn't write {filename}");
    }
    info!("wrote {filename} to disk");

    Ok(temp_path)
}

#[tokio::main]
async fn main() {
    Logger::try_with_env_or_str("info").unwrap()
        .start().unwrap();

    let yt_dlp_path = write_to_temp_file(YT_DLP, "yt-dlp.exe")
        .expect("Failed to write yt-dlp executable to a temp file");
    let ffmpeg_path = write_to_temp_file(FFMPEG, "ffmpeg.exe")
        .expect("Failed to write ffmpeg executable to a temp file");
    let downloader = Ytdlp { 
        ytdlp_path: yt_dlp_path.clone(),
        ffmpeg_path: ffmpeg_path.clone() 
    };

    let handles: Arc<Mutex<Vec<task::JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));

    let handles_clone = Arc::clone(&handles);
    let ui = ytdlrs::new().unwrap();
    ui.on_download(move |element, format, speed| {
        let downloader = downloader.clone();
        let handle = task::spawn(async move {
            downloader.download(&element, &format, &speed).await;
        });
        handles_clone.lock().unwrap().push(handle);
        return true;
    }); 
    ui.run().unwrap();

    let mut task = handles.lock().unwrap();
    for handle in task.iter_mut() {
        handle.await.unwrap();
    }

    std::fs::remove_file(yt_dlp_path)
        .expect("Failed to remove yt-dlp temp file");
    std::fs::remove_file(ffmpeg_path)
        .expect("Failed to remove yt-dlp temp file");
}

#[derive(Clone)]
struct Ytdlp {
    ytdlp_path: PathBuf,
    ffmpeg_path: PathBuf,
}


impl Ytdlp {
    pub async fn download(&self, element: &str, format: &str, speed: &str) {
        println!("scaricando serio {element}");
        println!("{format} in {speed}");
        let mut cmd = Command::new(&self.ytdlp_path);
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit());
        cmd.arg("--ffmpeg-location")
            .arg(&self.ffmpeg_path);

        match format {
            "video" => {
                match speed {
                    "veloce" => {
                        cmd.arg("--remux-video");
                    },
                    "lento" => {
                        cmd.arg("--recode-video");
                    },
                    _ => {
                        error!("couldn't recognize the speed");
                    }
                }
                cmd.arg("mp4");
            },
            "audio" => {
                cmd.arg("-f bestaudio")
                    .arg("-x")
                    .arg("--audio-format")
                    .arg("mp3");
            },
            _ => {
                error!("couldn't recognize the format");
            }
        }

        cmd.arg(&element);

        cmd.spawn().unwrap().wait().unwrap();
    }
}

