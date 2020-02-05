# rtp2wav

## Command for send rtp stream

`ffmpeg -re -i data/in.wav -acodec libopus -ac 1 -ar 48000 -f rtp rtp://127.0.0.1:34254/`

## `Cargo run` to start listening
