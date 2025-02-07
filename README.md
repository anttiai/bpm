# Broadcast Performance Metrics (BPM)
Library for collecting Broadcast Performance Metrics for AWS IVS Multitrack Streaming. Written in Rust, with support for C and C++ via a Foreign Function Interface (FFI).
The user should send metrics in-band via SEI (for AVC/HEVC) or OBU (AV1) messages on all video tracks just prior to the IDR. This library maintains internal state within single process.

## Concept
Integrate with encoding software such as FFmpeg or GStreamer. Call **bpm_frame_encoded** after successfully encoding a frame. Use **bpm_frame_lagged** and **bpm_frame_dropped** to track lagged and dropped frames, respectively. For keyframes, render and fetch metrics using **bpm_render_ts_ptr**, **bpm_render_sm_ptr**, and **bpm_render_erm_ptr**. Inject the returned data into SEI or OBU messages and free the memory with **bpm_destroy**.

## Build
```bash
cargo build --release
```

## Example in C
Build and run the C example
```bash
gcc -o build/example example.c -Ltarget/release/ -lbpm
./build/example
```