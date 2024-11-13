#![windows_subsystem = "windows"]
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use flexi_logger::{Age, Cleanup, Criterion, FileSpec, Logger, Naming};
use log::{error, info, trace};

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
        .log_to_file(FileSpec::default().directory(env::temp_dir()))
        .format(flexi_logger::detailed_format)
        .rotate(
            Criterion::Age(Age::Day), 
            Naming::Timestamps, 
            Cleanup::KeepLogFiles(5))
        .start().unwrap();

    let yt_dlp_path = write_to_temp_file(YT_DLP, "yt-dlp.exe")
        .expect("Failed to write yt-dlp executable to a temp file");
    let ffmpeg_path = write_to_temp_file(FFMPEG, "ffmpeg.exe")
        .expect("Failed to write ffmpeg executable to a temp file");
    let downloader = Ytdlp { 
        ytdlp_path: yt_dlp_path.clone(),
        ffmpeg_path: ffmpeg_path.clone() 
    };
    downloader.update().await;

    let handles: Arc<Mutex<Option<task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));

    let handles_clone = Arc::clone(&handles);
    let handles_clone_second = Arc::clone(&handles);
    let ui = ytdlrs::new().unwrap();
    ui.on_is_running(move || {
        let handle =  handles_clone_second.lock().unwrap();
        trace!("check start!");
        if !handle.is_some() {
            trace!("checking, handle is not some!");
            return false;
        }
        let process = handle.as_ref().unwrap();
        trace!("checking extracted process handle!");
        if process.is_finished() {
            trace!("checking, the process has finished!");
            return false;
        }
        return true;
    });
    ui.on_download(move |element, format, speed| {
        let downloader = downloader.clone();
        let handle = task::spawn(async move {
            for e in element.lines() {
                downloader.download(e, &format, &speed).await;
            }
        });
        *handles_clone.lock().unwrap() = Some(handle);
        return true;
    }); 
    ui.run().unwrap();

    let mut lock = handles.lock().unwrap();
    let process = lock.as_mut().unwrap();
    if !process.is_finished(){
        info!("process isn't finished, trying abort");
        process.abort();
        info!("aborted");
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
    pub async fn update(&self) {
        let _cmd = Command::new(&self.ytdlp_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x08000000)
            .arg("-U").output().unwrap();
    }
    pub async fn download(&self, element: &str, format: &str, speed: &str) {
        info!("scaricando serio {element}");
        info!("{format} in {speed}");
        let mut cmd = Command::new(&self.ytdlp_path);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(0x08000000);
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

        let out = cmd.output().unwrap();
        info!("status: {}", out.status);
        info!("output: {}, {}",
            String::from_utf8(out.stdout).unwrap(),
            String::from_utf8(out.stderr).unwrap());
    }
}

