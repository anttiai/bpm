use parking_lot::{Mutex};
use chrono::{Utc, SecondsFormat};

const UUID_TS: [u8; 16] = [ 0x0a, 0xec, 0xff, 0xe7, 0x52, 0x72, 0x4e, 0x2f, 0xa6, 0x2f, 0xd1, 0x9c, 0xd6, 0x1a, 0x93, 0xb5 ];
const UUID_SM: [u8; 16] = [ 0xca, 0x60, 0xe7, 0x1c, 0x6a, 0x8b, 0x43, 0x88, 0xa3, 0x77, 0x15, 0x1d, 0xf7, 0xbf, 0x8a, 0xc2 ];
const UUID_ERM: [u8; 16] = [ 0xf1, 0xfb, 0xc1, 0xd5, 0x10, 0x1e, 0x4f, 0xb5, 0xa6, 0x1e, 0xb8, 0xce, 0x3c, 0x07, 0xb8, 0xc0 ];

const NULL: u8 = 0;
const TS_TYPE: u8 = 1; // RFC3339
const _BPM_TS_EVENT_CTS: u8 = 1;  // Composition Time Event
const _BPM_TS_EVENT_FER: u8 = 2;  // Frame Encode Request Event
const _BPM_TS_EVENT_FERC: u8 = 3; // Frame Encode Request Complete
const BPM_TS_EVENT_PIR: u8 = 4;  // Packet Interleave Request Event

struct State {
    tracks: u32,

    // Session metrics
    sm_rendered: u32, // Frames rendered by compositor
    sm_lagged: u32,   // Frames lagged by compositor
    sm_dropped: u32,  // Frames dropped due to network congestion
    sm_output: u32,   // Sum of all video encoder rendition sinks

    // Encoded Rendition Metrics
    erm_input: Option<Vec<u32>>,   // Frames input to the encoder rendition
    erm_skipped: Option<Vec<u32>>, // Frames skipped by the encoder rendition
    erm_output: Option<Vec<u32>>,  // Frames output (encoded) by the encoder rendition
}

impl Default for State {
    fn default() -> State {
        State {
            tracks: 0,
            sm_rendered: 0,
            sm_lagged: 0,
            sm_dropped: 0,
            sm_output: 0,
            erm_input: None,
            erm_skipped: None,
            erm_output: None,
        }
    }
}

// Maintain global state
lazy_static::lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State::default());
}


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


#[no_mangle]
pub extern "C" fn bpm_frame_encoded(track: u32, is_keyframe: bool) -> u32 {
    let mut state = STATE.lock();

    // Spec: "The primary, highest quality video track must be packaged
    // and sent as enhanced RTMP single-track video packets" = track 0
    if track == 0 {
        state.sm_rendered += 1;
    }

    // All tracks update: Sum of all video encoder rendition sinks
    state.sm_output += 1;

    // Update encoded rendition metrics
    if let Some(ref mut input) = state.erm_input {
        input[track as usize] += 1;
    }

    // Update encoded rendition metrics
    if let Some(ref mut output) = state.erm_output {
        output[track as usize] += 1;
    }

    if is_keyframe {
        return 123;
    }
    return 0;
}


#[no_mangle]
pub extern "C" fn bpm_frame_lagged(track: u32) {
    let mut state = STATE.lock();
    state.sm_lagged += 1;

    // Update encoded rendition metrics
    if let Some(ref mut skipped) = state.erm_skipped {
        skipped[track as usize] += 1;
    }
}


#[no_mangle]
pub extern "C" fn bpm_frame_dropped(track: u32) {
    let mut state = STATE.lock();
    state.sm_dropped += 1;

    // Update encoded rendition metrics
    if let Some(ref mut skipped) = state.erm_skipped {
        skipped[track as usize] += 1;
    }
}


fn now_in_rfc3339() -> String {
    let now = Utc::now();
    let formatted_time = now.to_rfc3339_opts(SecondsFormat::Millis, true);
    return formatted_time;
}


fn construct_data(track: u32) -> [u8; 44] {

    // BPM TS (Timestamp) SE
    let mut ts_data: [u8; 44] = [0; 44];
    ts_data[0..16].copy_from_slice(&UUID_TS);
    ts_data[16] = 0x01;     // ts_reserved_zero_4bits & num_timestamps_minus1
    ts_data[17] = TS_TYPE;  // RFC3339
    ts_data[18] = BPM_TS_EVENT_PIR; // "IVS expects BPM SM SEI using timestamp_event only set to 4"
    ts_data[19..43].copy_from_slice(now_in_rfc3339().as_bytes()); // Current UTC time
    ts_data[43] = NULL;     // Null terminated string

    return ts_data;
}



#[no_mangle]
pub extern "C" fn bpm_print_state() {
    let state = STATE.lock();
    print!("Time: {}\n", now_in_rfc3339());
    print!("Tracks: {}\n", state.tracks);
    print!("Rendered: {}\n", state.sm_rendered);
    print!("Lagged: {}\n", state.sm_lagged);
    print!("Dropped: {}\n", state.sm_dropped);
    print!("Output: {}\n", state.sm_output);
    print!("Input: {:?}\n", state.erm_input);
    print!("Skipped: {:?}\n", state.erm_skipped);
    print!("Output: {:?}\n", state.erm_output);

    // Print data
    let data = construct_data(0);
    print!("Data: {:02X?}\n", data);
}
