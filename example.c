#include "bpm.h"
#include <stdio.h>
#include <stdbool.h>

void render_and_print_data(int track_idx) {
    // Pointers to the Rust-allocated memory
    uint8_t* ts_data = NULL;
    uint32_t ts_size = 0;
    bpm_render_ts_ptr(0, 0, 0, 0, &ts_data, &ts_size);
    printf("TS: ");
    for (int i=0; i<ts_size; i++) {
        printf("0x%02X ", ts_data[i]);
    }
    printf("\n");
    bpm_destroy(ts_data);

    uint8_t* sm_data = NULL;
    uint32_t sm_size = 0;
    bpm_render_sm_ptr(track_idx, &sm_data, &sm_size);
    printf("SM: ");
    for (int i=0; i<sm_size; i++) {
        printf("0x%02X ", sm_data[i]);
    }
    printf("\n");
    bpm_destroy(sm_data);

    uint8_t* erm_data = NULL;
    uint32_t erm_size = 0;
    bpm_render_erm_ptr(track_idx, &erm_data, &erm_size);
    printf("ERM: ");
    for (int i=0; i<erm_size; i++) {
        printf("0x%02X ", erm_data[i]);
    }
    printf("\n");
    bpm_destroy(erm_data);
}

int main() {
    // Two tracks
    int track0 = bpm_get_track_index("1080p60");
    int track1 = bpm_get_track_index("720p30");

    int frame = 0;
    do {
        frame++;

        // Track 0: Every frame encoded (60fps)
        bpm_frame_encoded(track0);

        // Track 1: Every other frame encoded (30fps)
        if (frame % 2 == 0) {
            bpm_frame_encoded(track1);
        }

        // Print state and data every 120 frames ("keyframe interval 2s")
        if (frame % 120 == 0) {
            printf("\n* Frame %d\n", frame);
            bpm_print_state();
            render_and_print_data(track0);
            render_and_print_data(track1);
        }
    } while (frame < 1000);
    return 0;
}