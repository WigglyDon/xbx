/* SPDX-License-Identifier: MIT
 * Developer-only host CPU-time aggregates. No guest state or instruction data.
 * The retained core owns execution; Rust owns enabling/reporting this candidate.
 */
#ifndef XBX_CPU_COST_H
#define XBX_CPU_COST_H
void xbx_cpu_cost_append(GString *out);
#ifdef XBX_CPU_COST_OWNER
#include <time.h>
typedef struct XbxCpuCostSample {
    bool selected;
    uint64_t time[6];
} XbxCpuCostSample;
static __thread uint32_t xbx_cpu_cost_stride;
static uint64_t xbx_cpu_cost_attempts, xbx_cpu_cost_completed;
static uint64_t xbx_cpu_cost_invalid_clock, xbx_cpu_cost_phase[5];
#ifndef XBX_COST_CLOCK
static uint64_t xbx_cpu_cost_clock(void)
{
    struct timespec ts;
    if (clock_gettime(CLOCK_THREAD_CPUTIME_ID, &ts) != 0) {
        return 0;
    }
    return (uint64_t)ts.tv_sec * 1000000000 + ts.tv_nsec;
}
#define XBX_COST_CLOCK xbx_cpu_cost_clock
#endif
static inline XbxCpuCostSample xbx_cpu_cost_begin(void)
{
    XbxCpuCostSample sample;
    sample.selected = (++xbx_cpu_cost_stride & 255) == 0;
    if (sample.selected) {
        qatomic_fetch_add(&xbx_cpu_cost_attempts, 1);
        sample.time[0] = XBX_COST_CLOCK();
    }
    return sample;
}
static inline void xbx_cpu_cost_mark(XbxCpuCostSample *sample, unsigned phase)
{
    if (sample->selected) {
        sample->time[phase] = XBX_COST_CLOCK();
    }
}
static inline void xbx_cpu_cost_finish(XbxCpuCostSample *sample)
{
    if (!sample->selected) {
        return;
    }
    xbx_cpu_cost_mark(sample, 5);
    for (unsigned i = 0; i < 5; i++) {
        if (!sample->time[i] || sample->time[i + 1] < sample->time[i]) {
            qatomic_fetch_add(&xbx_cpu_cost_invalid_clock, 1);
            return;
        }
    }
    for (unsigned i = 0; i < 5; i++) {
        qatomic_fetch_add(&xbx_cpu_cost_phase[i],
                          sample->time[i + 1] - sample->time[i]);
    }
    qatomic_fetch_add(&xbx_cpu_cost_completed, 1);
}
void xbx_cpu_cost_append(GString *out)
{
    g_string_append_printf(out,
        "XBX CPU sample stride 256\n"
        "XBX CPU attempts %" PRIu64 "\n"
        "XBX CPU completed %" PRIu64 "\n"
        "XBX CPU invalid clock %" PRIu64 "\n"
        "XBX CPU lookup ns %" PRIu64 "\n"
        "XBX CPU translation ns %" PRIu64 "\n"
        "XBX CPU prepare ns %" PRIu64 "\n"
        "XBX CPU execute ns %" PRIu64 "\n"
        "XBX CPU tail ns %" PRIu64 "\n",
        qatomic_read(&xbx_cpu_cost_attempts),
        qatomic_read(&xbx_cpu_cost_completed),
        qatomic_read(&xbx_cpu_cost_invalid_clock),
        qatomic_read(&xbx_cpu_cost_phase[0]),
        qatomic_read(&xbx_cpu_cost_phase[1]),
        qatomic_read(&xbx_cpu_cost_phase[2]),
        qatomic_read(&xbx_cpu_cost_phase[3]),
        qatomic_read(&xbx_cpu_cost_phase[4]));
}
#endif
#endif
