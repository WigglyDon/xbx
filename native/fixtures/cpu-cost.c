/* SPDX-License-Identifier: MIT
 * Original deterministic measurement boundary/abandoned-attempt controls.
 * Includes the exact candidate header, not a second counter implementation.
 */
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <inttypes.h>
#include <string.h>
#include <glib.h>
#define qatomic_read(p) __atomic_load_n((p), __ATOMIC_SEQ_CST)
#define qatomic_fetch_add(p, n) __atomic_fetch_add((p), (n), __ATOMIC_SEQ_CST)
static uint64_t clock_value = 100, clock_reads;
static uint64_t fixture_clock(void)
{
    clock_reads++;
    return clock_value += 100;
}
#define XBX_COST_CLOCK fixture_clock
#define XBX_CPU_COST_OWNER
#include SUBJECT
int main(void)
{
    uint64_t work = 0;
    for (unsigned i = 0; i < 1024; i++) {
        XbxCpuCostSample s = xbx_cpu_cost_begin();
        work += i;
        for (unsigned phase = 1; phase <= 4; phase++) {
            xbx_cpu_cost_mark(&s, phase);
        }
        xbx_cpu_cost_finish(&s);
    }
    assert(work == 523776 && clock_reads == 24);
    assert(xbx_cpu_cost_attempts == 4 && xbx_cpu_cost_completed == 4);
    for (unsigned i = 0; i < 5; i++) assert(xbx_cpu_cost_phase[i] == 400);
    /* A native longjmp/early exit abandons a stack-local sample. The next
       begin remains independent; no pointer into that stack is retained. */
    XbxCpuCostSample s = { 0 };
    for (unsigned i = 0; i < 256; i++) s = xbx_cpu_cost_begin();
    assert(s.selected && xbx_cpu_cost_attempts == 5);
    for (unsigned i = 0; i < 256; i++) {
        s = xbx_cpu_cost_begin();
        for (unsigned p = 1; p <= 4; p++) xbx_cpu_cost_mark(&s, p);
        xbx_cpu_cost_finish(&s);
    }
    assert(xbx_cpu_cost_attempts == 6 && xbx_cpu_cost_completed == 5);
    /* Failed/backwards clock observations contribute no partial timings. */
    for (unsigned i = 0; i < 256; i++) s = xbx_cpu_cost_begin();
    for (unsigned p = 1; p <= 4; p++) xbx_cpu_cost_mark(&s, p);
    s.time[2] = s.time[1] - 1;
    xbx_cpu_cost_finish(&s);
    assert(xbx_cpu_cost_invalid_clock == 1 && xbx_cpu_cost_completed == 5);
    for (unsigned i = 0; i < 5; i++) assert(xbx_cpu_cost_phase[i] == 500);
    GString *text = g_string_new(NULL);
    xbx_cpu_cost_append(text);
    assert(strstr(text->str, "XBX CPU attempts 7\n"));
    assert(strstr(text->str, "XBX CPU completed 5\n"));
    assert(strstr(text->str, "XBX CPU execute ns 500\n"));
    g_string_free(text, true);
    return 0;
}
