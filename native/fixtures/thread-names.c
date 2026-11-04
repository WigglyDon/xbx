/* SPDX-License-Identifier: MIT
 * Original public pthread-name boundary control. No game/device inputs.
 * SUBJECT contains the exact retained GPL-2.0-or-later thread entry function.
 */
#define _GNU_SOURCE
#include <assert.h>
#include <stdbool.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    void *(*start_routine)(void *);
    void *arg;
    char *name;
} QemuThreadArgs;
static bool name_threads;
static unsigned exits;
#ifdef HOST_CONFIG
#include HOST_CONFIG
#endif
#define QEMU_TSAN_ANNOTATE_THREAD_NAME(n) ((void)(n))
#define g_free free
static inline size_t g_strlcpy(char *to, const char *from, size_t capacity)
{
    size_t length = strlen(from);
    if (capacity) {
        size_t count = length < capacity - 1 ? length : capacity - 1;
        memcpy(to, from, count);
        to[count] = 0;
    }
    return length;
}
static void qemu_thread_atexit_notify(void *arg)
{
    assert(!arg);
    exits++;
}
#include SUBJECT

static const char *expected;
static bool matched;
static void *check_name(void *arg)
{
    assert(arg == &matched);
    char actual[16];
    assert(!pthread_getname_np(pthread_self(), actual, sizeof(actual)));
    matched = strcmp(actual, expected) == 0;
    return arg;
}
int main(int argc, char **argv)
{
    assert(argc == 3);
    assert(!pthread_setname_np(pthread_self(), "fixture-parent"));
    name_threads = strcmp(argv[1], "on") == 0;
    bool null_name = strcmp(argv[2], "null-name") == 0;
    char bounded[16];
    g_strlcpy(bounded, argv[2], sizeof(bounded));
    expected = name_threads && !null_name ? bounded : "fixture-parent";
    QemuThreadArgs *args = calloc(1, sizeof(*args));
    assert(args);
    args->name = null_name ? NULL : strdup(argv[2]);
    args->start_routine = check_name;
    args->arg = &matched;
    pthread_t thread;
    assert(!pthread_create(&thread, NULL, qemu_thread_start, args));
    void *result;
    assert(!pthread_join(thread, &result));
    assert(result == &matched && exits == 1);
    printf("role=%s enabled=%d matched=%d clean_join=1\n", argv[2], name_threads, matched);
    return matched ? 0 : 1;
}
