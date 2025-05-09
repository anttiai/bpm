#include <stdint.h>

#ifndef BPM_H
#define BPM_H

#ifdef __cplusplus
extern "C" {
#endif

int bpm_get_track_index(const char* track_fingerprint);
void bpm_frame_encoded(int track_idx);
void bpm_frame_lagged(int track_idx);
void bpm_frame_dropped(int track_idx);
uint8_t bpm_render_ts_ptr(int ts_cts, int ts_fer, int ts_ferc, int ts_pir, uint8_t** ts_data, uint32_t* ts_size);
uint8_t bpm_render_sm_ptr(int track_idx, uint8_t** ts_data, uint32_t* ts_size);
uint8_t bpm_render_erm_ptr(int track_idx, uint8_t** ts_data, uint32_t* ts_size);
void bpm_destroy(uint8_t* data);
void bpm_print_state(void);

#ifdef __cplusplus
}
#endif

#endif