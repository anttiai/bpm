#include <stdint.h>

#ifndef BPM_H
#define BPM_H

#ifdef __cplusplus
extern "C" {
#endif

void bpm_frame_encoded_c(const char* track_fp);
void bpm_frame_lagged_c(const char* track_fp);
void bpm_frame_dropped_c(const char* track_fp);
uint8_t bpm_ts_ptr(uint8_t** ts_data, uint32_t* ts_size);
void bpm_destroy(uint8_t* data);
void bpm_print_state();

#ifdef __cplusplus
}
#endif

#endif