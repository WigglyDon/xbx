/* SPDX-License-Identifier: MIT
 * Original two-bank inputs for the exact retained LGPL state decoder.
 * No game, firmware implementation, GPU or private state is a fixture.
 */
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <stdio.h>
#include <inttypes.h>
#include <math.h>
static unsigned ctz32(uint32_t v) { return (unsigned)__builtin_ctz(v); }
#include REGS
typedef uint64_t hwaddr;
typedef struct PGRAPHState { unsigned unused; } PGRAPHState;
typedef struct MemoryRegion { uint64_t size; } MemoryRegion;
typedef struct NV2AState {
    PGRAPHState pgraph;
    struct { uint32_t regs[0x1000]; } pvideo;
    MemoryRegion *vram;
} NV2AState;
#define container_of(p, type, member) ((type *)((char *)(p) - offsetof(type, member)))
#define qatomic_read(p) (*(p))
static unsigned warnings;
#define warn_report_once(...) (++warnings)
static uint64_t memory_region_size(MemoryRegion *r) { return r->size; }
#include SUBJECT
static void setup(NV2AState *d, unsigned bank)
{
    unsigned b = bank * 4;
    unsigned width = bank ? 32 : 16, height = bank ? 16 : 8;
    d->pvideo.regs[NV_PVIDEO_BASE + b] = bank ? 0x8000 : 0x1000;
    d->pvideo.regs[NV_PVIDEO_LIMIT + b] = bank ? 0x300000 : 0x200000;
    d->pvideo.regs[NV_PVIDEO_OFFSET + b] = bank ? 0x80 : 0x40;
    d->pvideo.regs[NV_PVIDEO_SIZE_IN + b] = height << 16 | width;
    d->pvideo.regs[NV_PVIDEO_SIZE_OUT + b] = height << 16 | width;
    d->pvideo.regs[NV_PVIDEO_POINT_IN + b] = bank ? (16u << 17) | 32u : 0;
    d->pvideo.regs[NV_PVIDEO_POINT_OUT + b] = bank ? (4u << 16) | 3u : 0;
    d->pvideo.regs[NV_PVIDEO_DS_DX + b] = bank ? NV_PVIDEO_DIN_DOUT_UNITY / 2 : NV_PVIDEO_DIN_DOUT_UNITY;
    d->pvideo.regs[NV_PVIDEO_DT_DY + b] = bank ? NV_PVIDEO_DIN_DOUT_UNITY / 2 : NV_PVIDEO_DIN_DOUT_UNITY;
    d->pvideo.regs[NV_PVIDEO_FORMAT + b] = (bank ? 64u : 32u) |
        (NV_PVIDEO_FORMAT_COLOR_LE_CR8YB8CB8YA8 << ctz32(NV_PVIDEO_FORMAT_COLOR)) |
        (bank ? NV_PVIDEO_FORMAT_DISPLAY : 0);
    d->pvideo.regs[NV_PVIDEO_COLOR_KEY] = 0xff123456;
}
int main(int argc, char **argv)
{
    if (argc != 2) return 2;
    unsigned which = (unsigned)strtoul(argv[1], NULL, 10);
    if (which > 8) return 2;
    MemoryRegion ram = { .size = 64u * 1024u * 1024u };
    NV2AState d = { .vram = &ram };
    setup(&d, 0); setup(&d, 1);
    bool bank1 = which == 1 || which == 3 || which >= 6;
    bool enabled = which < 5;
    d.pvideo.regs[NV_PVIDEO_BUFFER] = bank1 ? NV_PVIDEO_BUFFER_1_USE : NV_PVIDEO_BUFFER_0_USE;
    if (which == 2) d.pvideo.regs[NV_PVIDEO_BUFFER] |= NV_PVIDEO_BUFFER_1_USE;
    if (which == 3) d.pvideo.regs[NV_PVIDEO_SIZE_IN] = UINT32_MAX;
    if (which == 4) d.pvideo.regs[NV_PVIDEO_SIZE_IN + 4] = UINT32_MAX;
    if (which == 5) d.pvideo.regs[NV_PVIDEO_BUFFER] = 0;
    if (which == 6) d.pvideo.regs[NV_PVIDEO_FORMAT + 4] &= ~NV_PVIDEO_FORMAT_PITCH;
    if (which == 7) d.pvideo.regs[NV_PVIDEO_BASE + 4] = (uint32_t)ram.size;
    if (which == 8) d.pvideo.regs[NV_PVIDEO_FORMAT + 4] = 0;
#ifdef AFTER
    uint32_t sampled;
    PvideoState state = get_pvideo_state(&d.pgraph, &sampled);
    assert(sampled == d.pvideo.regs[NV_PVIDEO_BUFFER]);
#else
    PvideoState state = get_pvideo_state(&d.pgraph);
#endif
    if (state.enabled != enabled) return 1;
    if (!enabled) return 0;
    assert(state.base == (bank1 ? 0x8000u : 0x1000u));
    assert(state.limit == (bank1 ? 0x300000u : 0x200000u));
    assert(state.offset == (bank1 ? 0x80u : 0x40u));
    assert(state.pitch == (bank1 ? 64 : 32));
    assert(state.in_width == (bank1 ? 32 : 16) && state.out_width == state.in_width);
    assert(state.in_height == (bank1 ? 16 : 8) && state.out_height == state.in_height);
    assert(state.in_s == (bank1 ? 32 : 0) && state.in_t == (bank1 ? 16 : 0));
    assert(state.out_x == (bank1 ? 3 : 0) && state.out_y == (bank1 ? 4 : 0));
    assert(state.scale_x == (bank1 ? 17.0f / 32.0f : 1.0f));
    assert(state.scale_y == (bank1 ? 9.0f / 16.0f : 1.0f));
    assert(state.color_key_enabled == bank1 && state.color_key == 0x123456);
    return 0;
}
