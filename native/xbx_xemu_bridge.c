/* SPDX-License-Identifier: GPL-2.0-or-later
 * Mechanical entry shim adapted from izzy2lost/xemu at 4967efd3,
 * android/app/src/main/cpp/xemu_android.cpp (retained GPL lineage).
 * Rust owns profile, launch and lifetime policy. QEMU owns its internal threads.
 */
#include <SDL3/SDL.h>
#include <jni.h>
#include <glib.h>
#include <stdatomic.h>
#include "xemu-settings.h"

extern void qemu_init(int argc, char **argv);
extern void xbx_force_char_stdio_link(void);
extern void xbx_force_lpc47m157_link(void);
extern int (*qemu_main)(void);
extern void xemu_android_display_preinit(void);
extern bool xemu_android_display_poll_ready(void);
extern bool xemu_android_exit_requested(void);
extern uint32_t xemu_android_guest_flips(void);
extern int xemu_machine_init_status(void);
extern void xemu_android_request_exit(void);
extern void xemu_android_display_loop(void);
extern void xemu_android_set_qemu_thread_finished(bool finished);
extern void xemu_android_pause_emulation(void);
extern void xemu_android_resume_emulation(void);
extern void xemu_android_set_display_mode_setting(int mode);

#define XBX_EXPORT __attribute__((visibility("default")))
static uint64_t dvd_offset;
static bool create_eeprom;
static bool (*resume_gate)(void);
static SDL_AtomicInt cancelled;

XBX_EXPORT int xbx_engine_abi(void) { return 3; }
XBX_EXPORT uint64_t xbx_engine_dvd_offset(void) { return dvd_offset; }
XBX_EXPORT bool xbx_engine_allow_eeprom_create(void) { return create_eeprom; }
XBX_EXPORT bool xbx_engine_resume_allowed(void) { return resume_gate && resume_gate(); }

XBX_EXPORT int xbx_engine_prepare(const char *config, uint64_t media_offset, bool allow_create, bool (*gate)(void))
{
    xbx_force_char_stdio_link();
    xbx_force_lpc47m157_link();
    dvd_offset = media_offset;
    create_eeprom = allow_create;
    resume_gate = gate;
    SDL_SetHintWithPriority(SDL_HINT_AUDIO_DRIVER, "aaudio,openslES", SDL_HINT_OVERRIDE);
    if (!SDL_Init(SDL_INIT_VIDEO | SDL_INIT_GAMEPAD)) {
        return -1;
    }
    xemu_settings_set_path(config);
    if (!xemu_settings_load()) {
        return -2;
    }
    xemu_android_set_qemu_thread_finished(false);
    xemu_android_set_display_mode_setting(1);
    xemu_android_display_preinit();
    return 0;
}

XBX_EXPORT int xbx_engine_run(int argc, char **argv)
{
    qemu_init(argc, argv);
    if (xemu_machine_init_status() != 1 || SDL_GetAtomicInt(&cancelled)) {
        return 1;
    }
    return qemu_main();
}

XBX_EXPORT int xbx_engine_status(void) { return xemu_machine_init_status(); }
XBX_EXPORT bool xbx_engine_display_poll_ready(void) { return xemu_android_display_poll_ready(); }
XBX_EXPORT bool xbx_engine_exit_requested(void) { return xemu_android_exit_requested(); }
XBX_EXPORT uint32_t xbx_engine_guest_flips(void) { return xemu_android_guest_flips(); }
XBX_EXPORT void xbx_engine_stop(void)
{
    SDL_SetAtomicInt(&cancelled, 1);
    if (xemu_machine_init_status() == 1) xemu_android_request_exit();
}
XBX_EXPORT void xbx_engine_display_loop(void) { xemu_android_display_loop(); }
XBX_EXPORT void xbx_engine_thread_finished(void) { xemu_android_set_qemu_thread_finished(true); }
XBX_EXPORT void xbx_engine_pause(void) { xemu_android_pause_emulation(); }
XBX_EXPORT void xbx_engine_resume(void) { xemu_android_resume_emulation(); }

/* Mechanical platform calls; selection, identity, layout and input policy are Rust. */
typedef struct XbxCallbacks {
    void (*media)(int, const char *, const char *);
    void (*touch)(int, int, float, float, float, float, float);
    void (*layout)(int, float, float, float, float *);
    const char *(*label)(int);
    void (*neutral)(void);
} XbxCallbacks;
static _Atomic(const XbxCallbacks *) callbacks;
XBX_EXPORT void xbx_platform_callbacks(const XbxCallbacks *value)
{ atomic_store_explicit(&callbacks, value, memory_order_release); }

XBX_EXPORT void xbx_platform_open(const char *saved_uri)
{
    JNIEnv *env = SDL_GetAndroidJNIEnv();
    jobject activity = SDL_GetAndroidActivity();
    jclass cls = (*env)->GetObjectClass(env, activity);
    jmethodID method = (*env)->GetMethodID(env, cls, "openDocument", "(Ljava/lang/String;)V");
    jstring uri = saved_uri ? (*env)->NewStringUTF(env, saved_uri) : NULL;
    (*env)->CallVoidMethod(env, activity, method, uri);
    if (uri) (*env)->DeleteLocalRef(env, uri);
    (*env)->DeleteLocalRef(env, cls);
    (*env)->DeleteLocalRef(env, activity);
}

XBX_EXPORT void xbx_platform_status(const char *text)
{
    JNIEnv *env = SDL_GetAndroidJNIEnv();
    jobject activity = SDL_GetAndroidActivity();
    jclass cls = (*env)->GetObjectClass(env, activity);
    jmethodID method = (*env)->GetMethodID(env, cls, "showStatus", "(Ljava/lang/String;)V");
    jstring value = (*env)->NewStringUTF(env, text);
    (*env)->CallVoidMethod(env, activity, method, value);
    (*env)->DeleteLocalRef(env, value);
    (*env)->DeleteLocalRef(env, cls);
    (*env)->DeleteLocalRef(env, activity);
}

JNIEXPORT void JNICALL Java_com_machinerhythm_xbx_XbxActivity_nativeDocument(
    JNIEnv *env, jclass cls, jint fd, jstring uri, jstring error)
{
    const char *u = uri ? (*env)->GetStringUTFChars(env, uri, NULL) : NULL;
    const char *e = error ? (*env)->GetStringUTFChars(env, error, NULL) : NULL;
    const XbxCallbacks *cb = atomic_load_explicit(&callbacks, memory_order_acquire);
    if (cb) cb->media(fd, u, e);
    if (u) (*env)->ReleaseStringUTFChars(env, uri, u);
    if (e) (*env)->ReleaseStringUTFChars(env, error, e);
}

JNIEXPORT void JNICALL Java_com_machinerhythm_xbx_XbxActivity_nativeTouch(
    JNIEnv *env, jclass cls, jint id, jint phase, jfloat x, jfloat y, jfloat w, jfloat h, jfloat density)
{
    const XbxCallbacks *cb = atomic_load_explicit(&callbacks, memory_order_acquire);
    if (cb) cb->touch(id, phase, x, y, w, h, density);
}

JNIEXPORT jfloatArray JNICALL Java_com_machinerhythm_xbx_XbxActivity_nativeLayout(
    JNIEnv *env, jclass cls, jint index, jfloat w, jfloat h, jfloat density)
{
    float out[5] = {0};
    const XbxCallbacks *cb = atomic_load_explicit(&callbacks, memory_order_acquire);
    if (cb) cb->layout(index, w, h, density, out);
    jfloatArray result = (*env)->NewFloatArray(env, 5);
    (*env)->SetFloatArrayRegion(env, result, 0, 5, out);
    return result;
}

JNIEXPORT jstring JNICALL Java_com_machinerhythm_xbx_XbxActivity_nativeLabel(JNIEnv *env, jclass cls, jint index)
{
    const XbxCallbacks *cb = atomic_load_explicit(&callbacks, memory_order_acquire);
    return (*env)->NewStringUTF(env, cb ? cb->label(index) : "");
}

JNIEXPORT void JNICALL Java_com_machinerhythm_xbx_XbxActivity_nativeNeutral(JNIEnv *env, jclass cls)
{
    const XbxCallbacks *cb = atomic_load_explicit(&callbacks, memory_order_acquire);
    if (cb) cb->neutral();
}

XBX_EXPORT void *xbx_digest_new(void) { return g_checksum_new(G_CHECKSUM_SHA256); }
XBX_EXPORT void xbx_digest_update(void *digest, const unsigned char *data, size_t size) { g_checksum_update(digest, data, size); }
XBX_EXPORT const char *xbx_digest_string(void *digest) { return g_checksum_get_string(digest); }
XBX_EXPORT void xbx_digest_free(void *digest) { g_checksum_free(digest); }

static _Atomic(SDL_Joystick *) touch_pad;
XBX_EXPORT int xbx_pad_create(void)
{
    SDL_VirtualJoystickDesc desc;
    SDL_INIT_INTERFACE(&desc);
    desc.type = SDL_JOYSTICK_TYPE_GAMEPAD;
    desc.name = "xbx touch";
    desc.naxes = 6;
    desc.nbuttons = 15;
    desc.axis_mask = (1u << 6) - 1;
    desc.button_mask = (1u << 15) - 1;
    SDL_JoystickID id = SDL_AttachVirtualJoystick(&desc);
    SDL_Joystick *pad = SDL_OpenJoystick(id);
    if (!pad || !SDL_IsGamepad(id)) return -1;
    SDL_SetJoystickVirtualAxis(pad, 4, -32768);
    SDL_SetJoystickVirtualAxis(pad, 5, -32768);
    char guid[33];
    SDL_GUIDToString(SDL_GetJoystickGUID(pad), guid, sizeof(guid));
    xemu_settings_set_string(&g_config.input.bindings.port1, guid);
    atomic_store_explicit(&touch_pad, pad, memory_order_release);
    return 0;
}

XBX_EXPORT void xbx_pad_apply(unsigned int buttons, const short *axes)
{
    SDL_Joystick *pad = atomic_load_explicit(&touch_pad, memory_order_acquire);
    if (!pad) return;
    SDL_LockJoysticks();
    for (int i = 0; i < 15; i++) SDL_SetJoystickVirtualButton(pad, i, (buttons & (1u << i)) != 0);
    for (int i = 0; i < 4; i++) SDL_SetJoystickVirtualAxis(pad, i, axes[i]);
    for (int i = 4; i < 6; i++) SDL_SetJoystickVirtualAxis(pad, i, (int)axes[i] * 2 - 32768);
    SDL_UnlockJoysticks();
}
