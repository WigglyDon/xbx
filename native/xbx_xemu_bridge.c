/* SPDX-License-Identifier: GPL-2.0-or-later
 * Mechanical entry shim adapted from izzy2lost/xemu at 4967efd3,
 * android/app/src/main/cpp/xemu_android.cpp (retained GPL lineage).
 * Rust owns profile, launch and lifetime policy. QEMU owns its internal threads.
 */
#include <SDL3/SDL.h>
#include "xemu-settings.h"

extern void qemu_init(int argc, char **argv);
extern int (*qemu_main)(void);
extern void xemu_android_display_preinit(void);
extern void xemu_android_display_wait_ready(void);
extern void xemu_android_display_loop(void);
extern void xemu_android_set_qemu_thread_finished(bool finished);
extern void xemu_android_pause_emulation(void);
extern void xemu_android_resume_emulation(void);

#define XBX_EXPORT __attribute__((visibility("default")))
static uint64_t dvd_offset;

XBX_EXPORT int xbx_engine_abi(void) { return 2; }
XBX_EXPORT uint64_t xbx_engine_dvd_offset(void) { return dvd_offset; }

XBX_EXPORT int xbx_engine_prepare(const char *config, uint64_t media_offset)
{
    dvd_offset = media_offset;
    SDL_SetHintWithPriority(SDL_HINT_AUDIO_DRIVER, "aaudio,openslES", SDL_HINT_OVERRIDE);
    if (!SDL_Init(SDL_INIT_VIDEO | SDL_INIT_GAMEPAD)) {
        return -1;
    }
    xemu_settings_set_path(config);
    if (!xemu_settings_load()) {
        return -2;
    }
    xemu_android_set_qemu_thread_finished(false);
    xemu_android_display_preinit();
    return 0;
}

XBX_EXPORT int xbx_engine_run(int argc, char **argv)
{
    qemu_init(argc, argv);
    return qemu_main();
}

XBX_EXPORT void xbx_engine_display_wait_ready(void) { xemu_android_display_wait_ready(); }
XBX_EXPORT void xbx_engine_display_loop(void) { xemu_android_display_loop(); }
XBX_EXPORT void xbx_engine_thread_finished(void) { xemu_android_set_qemu_thread_finished(true); }
XBX_EXPORT void xbx_engine_pause(void) { xemu_android_pause_emulation(); }
XBX_EXPORT void xbx_engine_resume(void) { xemu_android_resume_emulation(); }
