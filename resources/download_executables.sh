#!/bin/env bash

mkdir ffmpeg
cd ffmpeg
wget "https://www.gyan.dev/ffmpeg/builds/ffmpeg-git-full.7z"
7z x ffmpeg-git-full.7z
find -iname "ffmpeg.exe" -exec cp {} .. \;
cd ..
rm -rf ffmpeg

wget "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
