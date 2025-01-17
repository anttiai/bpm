use parking_lot::{Mutex};

struct State {
    tracks: u8,

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
pub extern "C" fn bpm_init(number_of_tracks: u8) {
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


#[no_mangle]
pub extern "C" fn bpm_print_state() {
    let state = STATE.lock();
    print!("Tracks: {}\n", state.tracks);
    print!("Rendered: {}\n", state.sm_rendered);
    print!("Lagged: {}\n", state.sm_lagged);
    print!("Dropped: {}\n", state.sm_dropped);
    print!("Output: {}\n", state.sm_output);
    print!("Input: {:?}\n", state.erm_input);
    print!("Skipped: {:?}\n", state.erm_skipped);
    print!("Output: {:?}\n", state.erm_output);
}
