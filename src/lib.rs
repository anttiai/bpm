use parking_lot::{Mutex};
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use core::panic;
use std::{ffi::CStr, os::raw::c_char, u32};

const MAX_OUTPUT_VIDEO_ENCODERS: usize = 6;

const SEI_UUID_SIZE: usize = 16;
const UUID_TS: [u8; SEI_UUID_SIZE] = [ 0x0a, 0xec, 0xff, 0xe7, 0x52, 0x72, 0x4e, 0x2f, 0xa6, 0x2f, 0xd1, 0x9c, 0xd6, 0x1a, 0x93, 0xb5 ];
const UUID_SM: [u8; SEI_UUID_SIZE] = [ 0xca, 0x60, 0xe7, 0x1c, 0x6a, 0x8b, 0x43, 0x88, 0xa3, 0x77, 0x15, 0x1d, 0xf7, 0xbf, 0x8a, 0xc2 ];
const UUID_ERM: [u8; SEI_UUID_SIZE] = [ 0xf1, 0xfb, 0xc1, 0xd5, 0x10, 0x1e, 0x4f, 0xb5, 0xa6, 0x1e, 0xb8, 0xce, 0x3c, 0x07, 0xb8, 0xc0 ];

const TS_TYPE_RFC3339: u8 = 1;
const NULL: u8 = 0;

const BPM_TS_EVENT_CTS: u8 = 1;         // Composition Time Event
const BPM_TS_EVENT_FER: u8 = 2;         // Frame Encode Request Event
const BPM_TS_EVENT_FERC: u8 = 3;        // Frame Encode Request Complete
const BPM_TS_EVENT_PIR: u8 = 4;         // Packet Interleave Request Event

const BPM_SM_FRAMES_RENDERED: u8 = 1;   // Frames rendered by compositor
const BPM_SM_FRAMES_LAGGED: u8 = 2;     // Frames lagged by compositor
const BPM_SM_FRAMES_DROPPED: u8 = 3;    // Frames dropped due to network congestion
const BPM_SM_FRAMES_OUTPUT: u8 = 4;     // Total frames output (sum of all video encoder rendition sinks)

const BPM_ERM_FRAMES_INPUT: u8 = 1;     // Frames input to the encoder rendition
const BPM_ERM_FRAMES_SKIPPED: u8 = 2;   // Frames skippped by the encoder rendition
const BPM_ERM_FRAMES_OUTPUT: u8 = 3;    // Frames output (encoded) by the encoder rendition


struct State {
    track_map: Vec<String>, // Track fingerprints for index in the metrics arrays

    // Session metrics
    sm_rendered: u32, // Frames rendered by compositor
    sm_lagged: u32,   // Frames lagged by compositor
    sm_dropped: u32,  // Frames dropped due to network congestion
    sm_output: u32,   // Sum of all video encoder rendition sinks

    // Values sent in the last metrics
    sm_rendered_ref: Vec<u32>,
    sm_lagged_ref: Vec<u32>,
    sm_dropped_ref: Vec<u32>,
    sm_output_ref: Vec<u32>,

    // Encoded Rendition Metrics
    erm_input: Vec<u32>,   // Frames input to the encoder rendition
    erm_skipped: Vec<u32>, // Frames skipped by the encoder rendition
    erm_output: Vec<u32>,  // Frames output (encoded) by the encoder rendition

    // Values sent in the last metrics
    erm_input_ref: Vec<u32>,
    erm_skipped_ref: Vec<u32>,
    erm_output_ref: Vec<u32>,
}

impl Default for State {
    fn default() -> State {
        State {
            track_map: Vec::new(),

            sm_rendered: 0,
            sm_lagged: 0,
            sm_dropped: 0,
            sm_output: 0,

            sm_rendered_ref: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
            sm_lagged_ref: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
            sm_dropped_ref: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
            sm_output_ref: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],

            erm_input: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
            erm_skipped: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
            erm_output: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],

            erm_input_ref: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
            erm_skipped_ref: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
            erm_output_ref: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
        }
    }
}

impl State {
    fn get_track_index(&mut self, fingerprint: String) -> usize {
        if let Some(index) = self.track_map.iter().position(|x| x == &fingerprint) {
            index
        } else {
            if self.track_map.len() >= MAX_OUTPUT_VIDEO_ENCODERS {
                panic!("Exceeded MAX_OUTPUT_VIDEO_ENCODERS limit");
            }
            self.track_map.push(fingerprint);
            self.track_map.len() - 1
        }
    }
}

// global state
lazy_static::lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State::default());
}


/// Get the index for the track by track fingerprint (e.g. codec_resolution_fps).
/// Used if the track index is not known by the encoder.
#[no_mangle]
pub extern "C" fn bpm_get_track_index(track_fp: *const c_char) -> i32 {
    if let Some(track_fp_str) = c_char_to_string(track_fp) {
        let mut state = STATE.lock();
        let track_idx = state.get_track_index(track_fp_str);
        return track_idx as i32;
    }
    return -1;
}

/// Frame encoded successfully
#[no_mangle]
pub extern "C" fn bpm_frame_encoded(track_idx: u32) {
    let mut state = STATE.lock();

    // Spec: "The primary, highest quality video track must be packaged
    // and sent as enhanced RTMP single-track video packets" = track 0
    if track_idx == 0 {
        state.sm_rendered += 1;
    }

    // All tracks: "Sum of all video encoder rendition sinks"
    state.sm_output += 1;
    // Frames input to the encoder rendition
    state.erm_input[track_idx as usize] += 1;
    // Frames output (encoded) by the encoder rendition
    state.erm_output[track_idx as usize] += 1;
}

/// Frame lagged while encoding
#[no_mangle]
pub extern "C" fn bpm_frame_lagged(track_idx: u32) {
    let mut state = STATE.lock();
    state.sm_lagged += 1;

    // Frames input to the encoder rendition
    state.erm_input[track_idx as usize] += 1;

    // Frames skipped by the encoder rendition
    state.erm_skipped[track_idx as usize] += 1;
}

/// Frame dropped due to network congestion
#[no_mangle]
pub extern "C" fn bpm_frame_dropped(track_idx: u32) {
    let mut state = STATE.lock();
    state.sm_dropped += 1;

    // Frames input to the encoder rendition
    state.erm_input[track_idx as usize] += 1;

    // Frames skipped by the encoder rendition
    state.erm_skipped[track_idx as usize] += 1;
}

/// BPM Timestamp
pub fn bpm_ts(ts_cts: i64, ts_fer: i64, ts_ferc: i64, ts_pir: i64) -> [u8; 125] {
    let now = now_in_rfc3339();
    let cts = if ts_cts > 0 { millis_in_rfc3339(ts_cts) } else { now.clone() };
    let fer = if ts_fer > 0 { millis_in_rfc3339(ts_fer) } else { now.clone() };
    let ferc = if ts_ferc > 0 { millis_in_rfc3339(ts_ferc) } else { now.clone() };
    let pir = if ts_pir > 0 { millis_in_rfc3339(ts_pir) } else { now.clone() };

    let mut ts_data: [u8; 125] = [0; 125];
    ts_data[0..16].copy_from_slice(&UUID_TS);
    ts_data[16] = 0x03;                                     // ts_reserved_zero_4bits & num_timestamps_minus1

    ts_data[17] = TS_TYPE_RFC3339;
    ts_data[18] = BPM_TS_EVENT_CTS;                         // Composition Time Event
    ts_data[19..43].copy_from_slice(cts.as_bytes());
    ts_data[43] = NULL;

    ts_data[44] = TS_TYPE_RFC3339;
    ts_data[45] = BPM_TS_EVENT_FER;                         // Frame Encode Request Event
    ts_data[46..70].copy_from_slice(fer.as_bytes());
    ts_data[70] = NULL;

    ts_data[71] = TS_TYPE_RFC3339;
    ts_data[72] = BPM_TS_EVENT_FERC;                        // Frame Encode Request Complete
    ts_data[73..97].copy_from_slice(ferc.as_bytes());
    ts_data[97] = NULL;

    ts_data[98] = TS_TYPE_RFC3339;
    ts_data[99] = BPM_TS_EVENT_PIR;                         // Packet Interleave Request Event
    ts_data[100..124].copy_from_slice(pir.as_bytes());
    ts_data[124] = NULL;

    return ts_data;
}

/// BPM Session Metrics
pub fn bpm_sm(track_idx: u32) -> [u8; 65] {
    let mut state = STATE.lock();
    let now = now_in_rfc3339();

    let mut sm_data: [u8; 65] = [0; 65];
    sm_data[0..16].copy_from_slice(&UUID_SM);
    sm_data[16] = 0x00;                                     // ts_reserved_zero_4bits & num_timestamps_minus1

    sm_data[17] = TS_TYPE_RFC3339;
    sm_data[18] = BPM_TS_EVENT_PIR;                         // "Amazon IVS expects BPM SM SEI using timestamp_event only set to 4 (BPM_TS_EVENT_PIR)"
    sm_data[19..43].copy_from_slice(now.as_bytes());
    sm_data[43] = NULL;

    sm_data[44] = 0x03;                                     // ts_reserved_zero_4bits & num_counters_minus1

    sm_data[45] = BPM_SM_FRAMES_RENDERED;
    sm_data[46..50].copy_from_slice(&state.sm_rendered_ref[track_idx as usize].to_be_bytes());
    sm_data[50] = BPM_SM_FRAMES_LAGGED;
    sm_data[51..55].copy_from_slice(&state.sm_lagged_ref[track_idx as usize].to_be_bytes());
    sm_data[55] = BPM_SM_FRAMES_DROPPED;
    sm_data[56..60].copy_from_slice(&state.sm_dropped_ref[track_idx as usize].to_be_bytes());
    sm_data[60] = BPM_SM_FRAMES_OUTPUT;
    sm_data[61..65].copy_from_slice(&state.sm_output_ref[track_idx as usize].to_be_bytes());

    state.sm_rendered_ref[track_idx as usize] = state.sm_rendered - state.sm_rendered_ref[track_idx as usize];
    state.sm_lagged_ref[track_idx as usize] = state.sm_lagged - state.sm_lagged_ref[track_idx as usize];
    state.sm_dropped_ref[track_idx as usize] = state.sm_dropped - state.sm_dropped_ref[track_idx as usize];
    state.sm_output_ref[track_idx as usize] = state.sm_output - state.sm_output_ref[track_idx as usize];

    return sm_data;
}

/// BPM Encoded Rendition Metrics
pub fn bpm_erm(track_idx: u32) -> [u8; 60] {
    let mut state = STATE.lock();
    let now = now_in_rfc3339();

    let mut erm_data: [u8; 60] = [0; 60];
    erm_data[0..16].copy_from_slice(&UUID_ERM);
    erm_data[16] = 0x00;                                     // ts_reserved_zero_4bits & num_timestamps_minus1

    erm_data[17] = TS_TYPE_RFC3339;
    erm_data[18] = BPM_TS_EVENT_PIR;                         // "Amazon IVS expects BPM ERM SEI using timestamp_event set only to 4 (BPM_TS_EVENT_PIR)."
    erm_data[19..43].copy_from_slice(now.as_bytes());
    erm_data[43] = NULL;

    erm_data[44] = 0x02;                                     // ts_reserved_zero_4bits & num_counters_minus1

    erm_data[45] = BPM_ERM_FRAMES_INPUT;
    erm_data[46..50].copy_from_slice(&state.erm_input_ref[track_idx as usize].to_be_bytes());
    erm_data[50] = BPM_ERM_FRAMES_SKIPPED;
    erm_data[51..55].copy_from_slice(&state.erm_skipped_ref[track_idx as usize].to_be_bytes());
    erm_data[55] = BPM_ERM_FRAMES_OUTPUT;
    erm_data[56..60].copy_from_slice(&state.erm_output_ref[track_idx as usize].to_be_bytes());

    state.erm_input_ref[track_idx as usize] = state.erm_input[track_idx as usize] - state.erm_input_ref[track_idx as usize];
    state.erm_skipped_ref[track_idx as usize] = state.erm_skipped[track_idx as usize] - state.erm_skipped_ref[track_idx as usize];
    state.erm_output_ref[track_idx as usize] = state.erm_output[track_idx as usize] - state.erm_output_ref[track_idx as usize];

    return erm_data;
}

/// Render BPM data and return a pointer to the data and its size.
/// Memory must be freed by the caller using bpm_destroy.
#[no_mangle]
pub extern "C" fn bpm_render_data_ptr(track_idx: u32, ts_data: *mut *mut u8, ts_size: *mut u32) -> i32 {
    if ts_data.is_null() || ts_size.is_null() {
        return -1;
    }

    let ts = bpm_ts(0, 0, 0, 0);
    let sm = bpm_sm(track_idx);
    let erm = bpm_erm(track_idx);

    const BPM_DATA_SIZE: usize = 250;
    let mut bpm_data: [u8; BPM_DATA_SIZE] = [0; BPM_DATA_SIZE];
    bpm_data[0..ts.len()].copy_from_slice(&ts);
    bpm_data[ts.len()..ts.len() + sm.len()].copy_from_slice(&sm);
    bpm_data[ts.len() + sm.len()..BPM_DATA_SIZE].copy_from_slice(&erm);

    let box_ptr = Box::new(bpm_data);
    unsafe {
        *ts_data = Box::into_raw(box_ptr) as *mut u8;
        *ts_size = bpm_data.len() as u32;
    }

    return 0;
}

/// Free the memory allocated by bpm_ts_ptr, bpm_erm_ptr, or bpm_sm_ptr
#[no_mangle]
pub extern "C" fn bpm_destroy(data: *mut u8) {
    if !data.is_null() {
        unsafe {
            let _ = Box::from_raw(data);
        }
    }
    else {
        panic!("Invalid pointer received");
    }
}

/// Print the state for debugging
#[no_mangle]
pub extern "C" fn bpm_print_state() {
    let state = STATE.lock();
    print!("Time: {}\n", now_in_rfc3339());
    print!("Track_map: {:?}\n", state.track_map);
    print!("SM Rendered: {}, {:?}\n", state.sm_rendered, state.sm_rendered_ref);
    print!("SM Lagged: {}, {:?}\n", state.sm_lagged, state.sm_lagged_ref);
    print!("SM Dropped: {}, {:?}\n", state.sm_dropped, state.sm_dropped_ref);
    print!("SM Output: {}, {:?}\n", state.sm_output, state.sm_output_ref);
    print!("ERM Input: {:?}, {:?}\n", state.erm_input, state.erm_input_ref);
    print!("ERM Skipped: {:?}, {:?}\n", state.erm_skipped, state.erm_skipped_ref);
    print!("ERM Output: {:?}, {:?}\n", state.erm_output, state.erm_output_ref);
}

/// Current time in RFC 3339 format
fn now_in_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Milliseconds in RFC 3339 format
fn millis_in_rfc3339(timestamp_ms: i64) -> String {
    let datetime: DateTime<Utc> = Utc.timestamp_millis_opt(timestamp_ms)
        .single()
        .expect("Invalid timestamp");
    datetime.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// C string to a Rust string
fn c_char_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        eprintln!("Error: Null pointer received");
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| {
            eprintln!("Error: Invalid UTF-8 string");
        })
        .ok()
}