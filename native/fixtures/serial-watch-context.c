/* SPDX-License-Identifier: MIT
 * Original host contract for unchanged retained UART/character-watch bodies.
 * GLib and socket backpressure are real; device declarations are controlled
 * stand-ins. This does not emulate an Android ABI or establish guest boot.
 */
#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <glib.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <unistd.h>

typedef gboolean (*FEWatchFunc)(void *, GIOCondition, void *);
typedef struct Chardev { GMainContext *gcontext; int fd; } Chardev;
typedef struct CharFrontend { Chardev *chr; } CharFrontend;
typedef struct ChardevClass {
    GSource *(*chr_add_watch)(Chardev *, GIOCondition);
} ChardevClass;
static GSource *add_watch(Chardev *chr, GIOCondition condition)
{
    GIOChannel *channel = g_io_channel_unix_new(chr->fd);
    GSource *source = g_io_create_watch(channel, condition);
    g_io_channel_unref(channel);
    return source;
}
static ChardevClass backend_class = { add_watch };
#define CHARDEV_GET_CLASS(unused) (&backend_class)
typedef struct Fifo8 { unsigned num; uint8_t data[16]; } Fifo8;
typedef struct SerialState {
    CharFrontend chr;
    uint8_t rbr, ier, iir, lcr, lsr, msr, mcr, scr, thr, tsr, fcr;
    unsigned tsr_retry, watch_tag, divider;
    GSource *watch_source;
    int thr_ipending, poll_msl, timeout_ipending, last_break_enable;
    void *fifo_timeout_timer, *modem_status_poll, *irq;
    uint64_t last_xmit_ts, char_transmit_time;
    Fifo8 recv_fifo, xmit_fifo;
} SerialState;
#define NANOSECONDS_PER_SECOND 1000000000ULL
#define QEMU_CLOCK_VIRTUAL 0
static uint64_t qemu_clock_get_ns(int clock) { (void)clock; return 1; }
static void timer_del(void *timer) { (void)timer; }
static void qemu_irq_lower(void *irq) { (void)irq; }
static void serial_update_irq(SerialState *s) { (void)s; }
static void serial_update_msl(SerialState *s) { (void)s; }
static void serial_receive1(SerialState *s, const uint8_t *b, int n)
{ (void)s; (void)b; (void)n; }
static bool fifo8_is_empty(Fifo8 *f) { return !f->num; }
static uint8_t fifo8_pop(Fifo8 *f) { assert(f->num); return f->data[--f->num]; }
static void fifo8_reset(Fifo8 *f) { f->num = 0; }
static int qemu_chr_fe_write(CharFrontend *c, const uint8_t *bytes, int n)
{ return write(c->chr->fd, bytes, n); }
static void serial_xmit(SerialState *s);
#include SUBJECT

int main(int argc, char **argv)
{
    struct rlimit no_core = {0, 0};
    assert(setrlimit(RLIMIT_CORE, &no_core) == 0);
    assert(argc == 2);
    bool custom = strcmp(argv[1], "default-reset") != 0;
    bool reset = strcmp(argv[1], "retry") != 0;
    GMainContext *context = custom ? g_main_context_new() : g_main_context_default();
    int pair[2];
    assert(socketpair(AF_UNIX, SOCK_STREAM | SOCK_NONBLOCK, 0, pair) == 0);
    int size = 4096;
    assert(setsockopt(pair[0], SOL_SOCKET, SO_SNDBUF, &size, sizeof(size)) == 0);
    char bytes[1024];
    memset(bytes, 'X', sizeof(bytes));
    while (write(pair[0], bytes, sizeof(bytes)) > 0) {}
    assert(errno == EAGAIN || errno == EWOULDBLOCK);
    Chardev chr = { context, pair[0] };
    SerialState s = { .chr = { &chr }, .thr = 0x5a };
    serial_xmit(&s);
    assert(s.tsr_retry == 1 && !(s.lsr & UART_LSR_TEMT));
    if (reset) serial_reset(&s);
    while (read(pair[1], bytes, sizeof(bytes)) > 0) {}
    assert(errno == EAGAIN || errno == EWOULDBLOCK);
    for (int i = 0; i < 8 && g_main_context_pending(context); i++) {
        g_main_context_iteration(context, false);
    }
    assert(s.lsr & UART_LSR_TEMT);
    assert(s.tsr_retry == 0);
    assert(!g_main_context_pending(context));
    if (!reset) {
        assert(read(pair[1], bytes, sizeof(bytes)) == 1 && bytes[0] == 0x5a);
    } else {
        assert(read(pair[1], bytes, sizeof(bytes)) < 0 && errno == EAGAIN);
    }
    close(pair[0]); close(pair[1]);
    if (custom) g_main_context_unref(context);
    puts("actual UART watch/reset/retry boundary passed; no guest executed");
    return 0;
}
