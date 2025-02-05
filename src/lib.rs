use parking_lot::{Mutex};
use chrono::{Utc, SecondsFormat};
use std::{ffi::CStr, os::raw::c_char};

// Maximum number of video encoder renditions
const MAX_OUTPUT_VIDEO_ENCODERS: usize = 6;

// Broadcast Performance Metrics SEI types
enum _bpm_sei_types {
	BPM_TS_SEI = 0, // BPM Timestamp SEI
	BPM_SM_SEI,     // BPM Session Metrics SEI
	BPM_ERM_SEI,    // BPM Encoded Rendition Metrics SEI
	BPM_MAX_SEI
}

const SEI_UUID_SIZE: usize = 16;
const UUID_TS: [u8; SEI_UUID_SIZE] = [ 0x0a, 0xec, 0xff, 0xe7, 0x52, 0x72, 0x4e, 0x2f, 0xa6, 0x2f, 0xd1, 0x9c, 0xd6, 0x1a, 0x93, 0xb5 ];
const _UUID_SM: [u8; SEI_UUID_SIZE] = [ 0xca, 0x60, 0xe7, 0x1c, 0x6a, 0x8b, 0x43, 0x88, 0xa3, 0x77, 0x15, 0x1d, 0xf7, 0xbf, 0x8a, 0xc2 ];
const _UUID_ERM: [u8; SEI_UUID_SIZE] = [ 0xf1, 0xfb, 0xc1, 0xd5, 0x10, 0x1e, 0x4f, 0xb5, 0xa6, 0x1e, 0xb8, 0xce, 0x3c, 0x07, 0xb8, 0xc0 ];

// Timestamp types
enum _bpm_ts_type {
	BPM_TS_RFC3339 = 1, // RFC3339 timestamp string
	BPM_TS_DURATION,    // Duration since epoch in milliseconds (64-bit)
	BPM_TS_DELTA        // Delta timestamp in nanoseconds (64-bit)
}

// Timestamp event tags
enum _BPM_TS_EVENT_FERCbpm_ts_event_tag {
	BPM_TS_EVENT_CTS = 1, // Composition Time Event
	BPM_TS_EVENT_FER,     // Frame Encode Request Event
	BPM_TS_EVENT_FERC,    // Frame Encode Request Complete Event
	BPM_TS_EVENT_PIR      // Packet Interleave Request Event
}

// Session Metrics types
enum _bpm_sm_type {
	BPM_SM_FRAMES_RENDERED = 1, // Frames rendered by compositor
	BPM_SM_FRAMES_LAGGED,       // Frames lagged by compositor
	BPM_SM_FRAMES_DROPPED,      // Frames dropped due to network congestion
	BPM_SM_FRAMES_OUTPUT        // Total frames output (sum of all video encoder rendition sinks)
}

// Encoded Rendition Metrics types
enum _bpm_erm_type {
	BPM_ERM_FRAMES_INPUT = 1, // Frames input to the encoder rendition
	BPM_ERM_FRAMES_SKIPPED,   // Frames skippped by the encoder rendition
	BPM_ERM_FRAMES_OUTPUT     // Frames output (encoded) by the encoder rendition
}


// Timestamp
const NULL: u8 = 0;
const TS_TYPE: u8 = 1;            // RFC3339
const _BPM_TS_EVENT_CTS: u8 = 1;  // Composition Time Event
const _BPM_TS_EVENT_FER: u8 = 2;  // Frame Encode Request Event
const _BPM_TS_EVENT_FERC: u8 = 3; // Frame Encode Request Complete
const BPM_TS_EVENT_PIR: u8 = 4;   // Packet Interleave Request Event


struct State {
    track_map: Vec<String>, // Track Quality -> index

    // Session metrics
    sm_rendered: u32, // Frames rendered by compositor
    sm_lagged: u32,   // Frames lagged by compositor
    sm_dropped: u32,  // Frames dropped due to network congestion
    sm_output: u32,   // Sum of all video encoder rendition sinks

    // Encoded Rendition Metrics
    erm_input: Vec<u32>,   // Frames input to the encoder rendition
    erm_skipped: Vec<u32>, // Frames skipped by the encoder rendition
    erm_output: Vec<u32>,  // Frames output (encoded) by the encoder rendition
}

impl Default for State {
    fn default() -> State {
        State {
            track_map: Vec::new(),
            sm_rendered: 0,
            sm_lagged: 0,
            sm_dropped: 0,
            sm_output: 0,
            erm_input: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
            erm_skipped: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
            erm_output: vec![0; MAX_OUTPUT_VIDEO_ENCODERS],
        }
    }
}

impl State {
    fn get_index(&mut self, fingerprint: String) -> usize {
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

/*
/// Initialize the BPM library
///
/// * `number_of_tracks` - Number of video tracks/renditions.
#[no_mangle]
pub extern "C" fn bpm_init(number_of_tracks: u32) {
    if number_of_tracks < 1 {
        panic!("Invalid number of tracks");
    }

    let mut state = STATE.lock();
    state.tracks = number_of_tracks;
    state.erm_input = Some(vec![0; number_of_tracks as usize]);
    state.erm_skipped = Some(vec![0; number_of_tracks as usize]);
    state.erm_output = Some(vec![0; number_of_tracks as usize]);
}
*/

/// Frame encoded successfully
fn bpm_frame_encoded(track_fp: String) {
    let mut state = STATE.lock();
    let track_idx = state.get_index(track_fp);

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

#[no_mangle]
pub extern "C" fn bpm_frame_encoded_c(track_fp: *const c_char) {
    if let Some(track_fp_str) = c_char_to_string(track_fp) {
        bpm_frame_encoded(track_fp_str);
    }
}


/// Frame lagged while encoding
fn bpm_frame_lagged(track_fp: String) {
    let mut state = STATE.lock();
    let track_idx = state.get_index(track_fp);
    state.sm_lagged += 1;

    // Frames input to the encoder rendition
    state.erm_input[track_idx as usize] += 1;

    // Frames skipped by the encoder rendition
    state.erm_skipped[track_idx as usize] += 1;
}

#[no_mangle]
pub extern "C" fn bpm_frame_lagged_c(track_fp: *const c_char) {
    if let Some(track_fp_str) = c_char_to_string(track_fp) {
        bpm_frame_lagged(track_fp_str);
    }
}


/// Frame was dropped due to network congestion
///
/// * `track_idx` - Track number.
fn bpm_frame_dropped(track_fp: String) {
    let mut state = STATE.lock();
    let track_idx = state.get_index(track_fp);
    state.sm_dropped += 1;

    // Frames input to the encoder rendition
    state.erm_input[track_idx as usize] += 1;

    // Frames skipped by the encoder rendition
    state.erm_skipped[track_idx as usize] += 1;
}

#[no_mangle]
pub extern "C" fn bpm_frame_dropped_c(track_fp: *const c_char) {
    if let Some(track_fp_str) = c_char_to_string(track_fp) {
        bpm_frame_dropped(track_fp_str);
    }
}


/// BPM TS (Timestamp)
pub fn bpm_ts() -> [u8; 44] {
    let mut ts_data: [u8; 44] = [0; 44];
    ts_data[0..16].copy_from_slice(&UUID_TS); // UUID
    ts_data[16] = 0x01;                            // ts_reserved_zero_4bits & num_timestamps_minus1
    ts_data[17] = TS_TYPE;                         // Current UTC time in RFC3339
    ts_data[18] = BPM_TS_EVENT_PIR;                // "IVS expects BPM SM SEI using timestamp_event only set to 4"
    ts_data[19..43].copy_from_slice(now_in_rfc3339().as_bytes()); // Current UTC time
    ts_data[43] = NULL;                            // Null termination
    return ts_data;
}


/// BPM TS pointer. Memory must be freed by the caller using bpm_destroy.
///
/// * `ts_data` - Pointer to the TS data.
/// * `ts_size` - Size of the TS data.
///
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn bpm_ts_ptr(ts_data: *mut *mut u8, ts_size: *mut u32) -> i32 {
    if ts_data.is_null() || ts_size.is_null() {
        return -1;
    }

    let ts = bpm_ts();
    let size = ts.len();
    let box_ptr = Box::new(ts);

    unsafe {
        *ts_data = Box::into_raw(box_ptr) as *mut u8;
        *ts_size = size as u32;
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
}


/// Print the state for debugging
#[no_mangle]
pub extern "C" fn bpm_print_state() {
    let state = STATE.lock();
    print!("Time: {}\n", now_in_rfc3339());
    print!("Track_map: {:?}\n", state.track_map);
    print!("SM Rendered: {}\n", state.sm_rendered);
    print!("SM Lagged: {}\n", state.sm_lagged);
    print!("SM Dropped: {}\n", state.sm_dropped);
    print!("SM Output: {}\n", state.sm_output);
    print!("ERM Input: {:?}\n", state.erm_input);
    print!("ERM Skipped: {:?}\n", state.erm_skipped);
    print!("ERM Output: {:?}\n", state.erm_output);

    let data = bpm_ts();
    print!("Data: {:02X?}\n", data);
}


fn now_in_rfc3339() -> String {
    let now = Utc::now();
    let formatted_time = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    return formatted_time;
}


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


fn main() {
    println!("Hello, world!");
    bpm_print_state();

    // Add some test data
    bpm_frame_encoded("track1".to_string());
    bpm_frame_encoded("track2".to_string());
    bpm_frame_lagged("track1".to_string());
    bpm_frame_lagged("track2".to_string());
    bpm_print_state();
}