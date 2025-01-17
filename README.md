# Broadcast Performance Metrics (BPM)
Broadcast Performance Metrics for IVS Multitrack Streaming and Twitch Enhanced Broadcasting. Written in Rust, with support for use in C and C++.

The metrics should be sent in-band via either SEI (for AVC/HEVC) or OBU (AV1) messages. BPM metrics must be inserted on all video tracks just prior to the IDR.

## Build
cargo build --release

## Use
Integrate with encoders such as FFmpeg or GStreamer. Call **bpm_frame_encoded** after encoding a frame successfully. For keyframes, inject the returned data into SEI or OBU messages. Use **bpm_frame_lagged** and **bpm_frame_dropped** to track lagged and dropped frames, respectively.

## Example in C
gcc -o build/example example.c -Ltarget/release/ -lbpm
./build/example