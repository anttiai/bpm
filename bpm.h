#include <stdbool.h>

#ifndef FOO_H
#define FOO_H

#ifdef __cplusplus
extern "C" {
#endif

void bpm_init(int value);
int bpm_frame_encoded(int track, bool is_keyframe);
void bpm_frame_lagged(int track);
void bpm_frame_dropped(int track);
void bpm_print_state();

#ifdef __cplusplus
}
#endif

#endif