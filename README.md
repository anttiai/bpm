# Broadcast Performance Metrics (BPM)
Library for collecting Broadcast Performance Metrics. Written in Rust, with support for C and C++ via a Foreign Function Interface (FFI).
The metrics should be sent in-band via either SEI (for AVC/HEVC) or OBU (AV1) messages and they must be inserted on all video tracks just prior to the IDR. This library maintains internal state within the same process.

## Build
cargo build --release

## Use
Integrate with software doing encoding, such as FFmpeg or GStreamer. Call **bpm_frame_encoded** after encoding a frame successfully. For keyframes, render and fetch metrics with **bpm_render_data_ptr** and inject the returned data into SEI or OBU messages. Use **bpm_frame_lagged** and **bpm_frame_dropped** to track lagged and dropped frames, respectively.

## Example in C
```bash
gcc -o build/example example.c -Ltarget/release/ -lbpm
./build/example
```