#include "bpm.h"
#include <stdio.h>

static void print_frame(int track, int frame_number, int data) {
    printf("Track %d, Frame %d, Data: %d\n", track, frame_number, data);
}

int main() {

    // Init with two tracks
    bpm_init(2);

    for (int i = 0; i < 130; i++) {

        // Keyframe interval 120 frames
        bool is_keyframe = (i % 120 == 0);

        // Track 0: Every frame encoded ("60fps")
        int data0 = bpm_frame_encoded(0, is_keyframe);
        print_frame(0, i, data0);

        // Track 1: Every other frame encoded ("30fps")
        if (i % 2 == 0) {
            int data1 = bpm_frame_encoded(1, is_keyframe);
            print_frame(1, i, data1);
        }
    }

    // Print state
    bpm_print_state();
    return 0;
}