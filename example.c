#include "bpm.h"
#include <stdio.h>
#include <stdbool.h>

static void print_frame(int track, int frame_number) {
    printf("Track %d, Frame %d\n", track, frame_number);
}

void render_and_print_data(int track_idx) {
    // Get data to a pointer holding the Rust-allocated memory
    uint8_t* data = NULL;
    uint32_t size = 0;
    int result = bpm_render_data_ptr(track_idx, &data, &size);
    for (int i = 0; i < size; i++) {
        printf("0x%02X ", data[i]);
    }
    bpm_destroy(data);
}

int main() {

    // Two tracks
    int track0 = bpm_get_track_index("1080p60");
    int track1 = bpm_get_track_index("720p30");

    // 1000 frames
    for (int i = 0; i < 1000; i++) {
        // Track 0: Every frame encoded (60fps)
        bpm_frame_encoded(track0);

        // Track 1: Every other frame encoded (30fps)
        if (i % 2 == 0) {
            bpm_frame_encoded(track1);
        }

        // Print state and data every 120 frames ("keyframe interval 2s")
        if (i % 120 == 0) {
            bpm_print_state();
            render_and_print_data(track0);
            render_and_print_data(track1);
        }
    }
    return 0;
}