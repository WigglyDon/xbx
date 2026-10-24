/* Mechanical test ABI only. The hash function is extracted verbatim from
 * the pinned LGPL-2.0-or-later PFIFO source by Rust xtask. */
#include <stdint.h>
#include "nv2a_regs.h"
#define ctz32(value) __builtin_ctz(value)
typedef struct { struct { uint32_t regs[0x4000]; } pfifo; } NV2AState;
#include "ramht-function.inc"

uint32_t xbx_fixture_ramht_hash(uint32_t size_field, uint32_t handle,
                              uint32_t channel)
{
    NV2AState d = {0};
    d.pfifo.regs[NV_PFIFO_RAMHT] = size_field << 16;
    d.pfifo.regs[NV_PFIFO_CACHE1_PUSH1] = channel;
    return ramht_hash(&d, handle);
}
