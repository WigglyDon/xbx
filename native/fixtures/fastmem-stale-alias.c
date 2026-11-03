/* SPDX-License-Identifier: MIT
 * Original synthetic mapping regression around retained GPL function bodies.
 * All memory/file descriptors belong to this process. No emulator/game input.
 */
#define _GNU_SOURCE
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <sys/mman.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <unistd.h>
#define PAGE_SZ 4096
#define WINDOW_SIZE (4 * PAGE_SZ)
#define MIN(a,b) ((a) < (b) ? (a) : (b))
static uintptr_t window_base;
static void *ram_host;
static uint64_t ram_size = 2 * PAGE_SZ;
static uint32_t aliases[2][2], (*rw_alias)[2] = aliases;
static uint8_t demote_count[2];
static unsigned long bits[1], *mapped_bits = bits;
static unsigned mapped_pages, stat_demotes;
static bool test_bit(unsigned i, unsigned long *b) { return (*b >> i) & 1; }
static bool test_and_clear_bit(unsigned i, unsigned long *b) {
    bool old = test_bit(i, b); *b &= ~(1ul << i); return old;
}
static void window_clear(void) { abort(); } /* Not used by these one-page cases. */
#include SUBJECT
static int expect_fault(volatile uint32_t *p, bool write) {
    pid_t child = fork();
    assert(child >= 0);
    if (!child) {
        if (write) *p = 7;
        else (void)*p;
        _exit(1);
    }
    int status;
    assert(waitpid(child, &status, 0) == child);
    return WIFSIGNALED(status) && WTERMSIG(status) == SIGSEGV ? 0 : 1;
}
int main(int argc, char **argv) {
    assert(argc == 2);
    struct rlimit no_core = {0, 0};
    assert(setrlimit(RLIMIT_CORE, &no_core) == 0);
    int fd = memfd_create("xbx-original-alias", 0);
    assert(fd >= 0 && ftruncate(fd, ram_size) == 0);
    ram_host = mmap(NULL, ram_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    assert(ram_host != MAP_FAILED);
    *(uint32_t *)ram_host = 0x524f4431;
    void *window = mmap(NULL, WINDOW_SIZE, PROT_NONE,
                        MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    assert(window != MAP_FAILED);
    window_base = (uintptr_t)window;
    volatile uint32_t *alias = (void *)(window_base + PAGE_SZ);
    assert(mmap((void *)alias, PAGE_SZ, PROT_READ | PROT_WRITE,
                MAP_SHARED | MAP_FIXED, fd, 0) == (void *)alias);
    assert(*alias == 0x524f4431);
    rw_alias[0][0] = 2;
    *mapped_bits = 1ul << 1;
    mapped_pages = 1;
    bool stale = strcmp(argv[1], "stale") == 0;
    assert(stale || strcmp(argv[1], "live") == 0);
    if (stale) xbox_fastmem_unmap_range(PAGE_SZ, PAGE_SZ);
    xbox_fastmem_reset_dirty((uintptr_t)ram_host, PAGE_SZ);
    assert(rw_alias[0][0] == 0);
    int result;
    if (stale) result = expect_fault(alias, false);
    else {
        assert(*alias == 0x524f4431);
        result = expect_fault(alias, true);
    }
    assert(munmap(window, WINDOW_SIZE) == 0);
    assert(munmap(ram_host, ram_size) == 0);
    assert(close(fd) == 0);
    return result;
}
