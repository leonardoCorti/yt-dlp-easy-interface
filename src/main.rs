#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::env;
use std::fs::File;
use std::io::{self, Write};

slint::slint!(export { ytdlrs } from "ui/main.slint";);

const YT_DLP: &[u8] = include_bytes!("../resources/yt-dlp.exe");
const FFMPEG: &[u8] = include_bytes!("../resources/ffmpeg.exe");

fn write_to_temp_file(data: &[u8], filename: &str) -> io::Result<std::path::PathBuf> {
    let temp_dir = env::temp_dir();
    let temp_path = temp_dir.join(filename);

    let mut file = File::create(&temp_path)?;
    file.write_all(data)?;

    Ok(temp_path)
}

fn main() {
    let yt_dlp_path = write_to_temp_file(YT_DLP, "yt-dlp.exe")
        .expect("Failed to write yt-dlp executable to a temp file");
    let ffmpeg_path = write_to_temp_file(FFMPEG, "ffmpeg.exe")
        .expect("Failed to write ffmpeg executable to a temp file");

    let ui = ytdlrs::new().unwrap();
    ui.on_download(|element, format, speed| {
        println!("scariculando {format} in {speed}");
        return true;
    }); 
    ui.run().unwrap();

    std::fs::remove_file(yt_dlp_path)
        .expect("Failed to remove yt-dlp temp file");
    std::fs::remove_file(ffmpeg_path)
        .expect("Failed to remove yt-dlp temp file");
}
