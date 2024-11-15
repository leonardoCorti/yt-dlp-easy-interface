# yt-dlp easy interface
this is a really barebone interface for yt-dlp made for a friend. 
It allows to download audio and video with yt-dlp, for maximum compatibility it produces mp3 and mp4, the video are only remuxed if the "veloce" option is checked so that it is quicker.

It supports only windows.

# build instructions
you will need to create a "resources" directory and put there ffmpeg.exe and yt-dlp.exe, then you can `cargo build` and it should work
