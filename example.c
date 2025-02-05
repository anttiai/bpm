#include "bpm.h"
#include <stdio.h>
#include <stdbool.h>

static void print_frame(int track, int frame_number) {
    printf("Track %d, Frame %d\n", track, frame_number);
}

/*
    uint8_t buffer[16];  // Allocate buffer in C

    // Call the Rust function to fill the buffer
    fill_buffer(buffer, 16);
     */

int main() {

    // Init with two tracks
    //bpm_init(2);

do {
    for (int i = 0; i < 130; i++) {

        // Keyframe interval 120 frames
        bool is_keyframe = (i % 120 == 0);

        // Track 0: Every frame encoded ("60fps")
        bpm_frame_encoded_c("1920x1080");
        //print_frame(0, i);

        // Track 1: Every other frame encoded ("30fps")
        if (i % 2 == 0) {
            bpm_frame_encoded_c("1280x720");
            //print_frame(1, i);
        }
    }

    // Print state
    bpm_print_state();


    // Get data
    uint8_t* ts_data = NULL; // A pointer to hold the Rust-allocated memory
    uint32_t ts_size = 0;    // A variable to hold the size of the data
    int result = bpm_ts_ptr(&ts_data, &ts_size); // Pass a pointer to `ts_data`

    // Print the data
    /*for (int i = 0; i < ts_size; i++) {
        printf("0x%02X ", ts_data[i]);
    }*/
    bpm_destroy(ts_data);

} while (1);
    return 0;
}