# Broadcast Performance Metrics (BPM)
Lubrary to help collecting Broadcast Performance Metrics. Written in Rust, with support for use in C and C++ with Foreign Function Interface.
The metrics should be sent in-band via either SEI (for AVC/HEVC) or OBU (AV1) messages. BPM metrics must be inserted on all video tracks just prior to the IDR.

This library keeps internal state within the same process. Each track should have distinct fingerprint, for example, resolution, framerate, and codec

## Build
cargo build --release

## Use
Integrate with encoders such as FFmpeg or GStreamer. Call **bpm_frame_encoded_c** after encoding a frame successfully. For keyframes, inject the returned data into SEI or OBU messages. Use **bpm_frame_lagged_c** and **bpm_frame_dropped_c** to track lagged and dropped frames, respectively.

## Example in C
gcc -o build/example example.c -Ltarget/release/ -lbpm
./build/example