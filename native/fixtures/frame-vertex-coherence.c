/* SPDX-License-Identifier: MIT
 * Original data and separate owned buffers exercise the exact retained RAM
 * update function and frame-copy block. No GPU or game data is accessed.
 */
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#define TARGET_PAGE_SIZE 4096u
#define TARGET_PAGE_ALIGN(x) (((x)+4095)&~4095ull)
#define BITS_TO_LONGS(x) (((x)+63)/64)
#define VK_WHOLE_SIZE UINT64_MAX
#define MIN(a,b) ((a)<(b)?(a):(b))
#define MAX(a,b) ((a)>(b)?(a):(b))
#define nv2a_profile_inc_counter(...) ((void)0)
#define BUFFER_VERTEX_RAM 0
#define VK_FINISH_REASON_VERTEX_BUFFER_DIRTY 0
#define RAM_BYTES (3*TARGET_PAGE_SIZE)
typedef uint64_t hwaddr;
typedef uint64_t VkDeviceSize;
typedef struct { uint8_t *mapped; size_t buffer_size; } StorageBuffer;
typedef struct {
    StorageBuffer vertex_ram;
    VkDeviceSize vertex_ram_flush_min, vertex_ram_flush_max;
    VkDeviceSize vertex_ram_propagate_min, vertex_ram_propagate_max;
    bool vertex_ram_initialized;
    unsigned long *uploaded_bitmap;
} FrameStagingState;
typedef struct {
    int current_frame, num_active_frames;
    size_t bitmap_size;
    FrameStagingState frame_staging[3];
    unsigned long cb_vertex_pages[1];
} PGRAPHVkState;
typedef struct { PGRAPHVkState *vk_renderer_state; } PGRAPHState;
static void pgraph_vk_download_surfaces_in_range_if_dirty(PGRAPHState *pg, hwaddr off, VkDeviceSize size)
{ (void)pg; (void)off; (void)size; }
static unsigned long find_next_bit(unsigned long *bits, size_t end, size_t start)
{
    for (size_t i=start;i<end;i++) if (*bits & (1ul<<i)) return i;
    return end;
}
static void pgraph_vk_finish(PGRAPHState *pg, int reason)
{ (void)pg; (void)reason; assert(!"fixture has no pending GPU reads"); }
static StorageBuffer *get_staging_buffer(PGRAPHVkState *r, int kind)
{ (void)kind; return &r->frame_staging[r->current_frame].vertex_ram; }
static unsigned long *get_uploaded_bitmap(PGRAPHVkState *r)
{ return r->frame_staging[r->current_frame].uploaded_bitmap; }
static void bitmap_set(unsigned long *bits, size_t start, size_t count)
{ for (size_t i=start;i<start+count;i++) *bits |= 1ul<<i; }

#include SUBJECT

static uint8_t buffers[3][RAM_BYTES], expected[RAM_BYTES];
static unsigned long bitmaps[3];
static unsigned long copied;

static bool rotate(PGRAPHState *pg)
{
    PGRAPHVkState *r=pg->vk_renderer_state;
    if (r->num_active_frames>1) {
        subject_rotate(pg,(r->current_frame+1)%r->num_active_frames);
        FrameStagingState *fs=&r->frame_staging[r->current_frame];
        if (fs->vertex_ram_flush_min<fs->vertex_ram_flush_max)
            copied+=fs->vertex_ram_flush_max-fs->vertex_ram_flush_min;
    }
    return !memcmp(buffers[r->current_frame],expected,RAM_BYTES);
}

static void update(PGRAPHState *pg, unsigned offset, uint8_t value)
{
    PGRAPHVkState *r=pg->vk_renderer_state;
    uint8_t before[3][RAM_BYTES];
    memcpy(before,buffers,sizeof(before));
    memset(expected+offset,value,16);
    /* A completed prior draw must not be mistaken for an overlapping
     * in-flight draw in this ownership-only test. */
    r->cb_vertex_pages[0]=0;
    pgraph_vk_update_vertex_ram_buffer(pg,offset,expected+offset,16);
    for(int i=0;i<3;i++) if(i!=r->current_frame)
        assert(!memcmp(before[i],buffers[i],RAM_BYTES));
}

int main(int argc,char **argv)
{
    assert(argc==2);
    bool one=!strcmp(argv[1],"one"), two=!strcmp(argv[1],"two");
    bool three=!strcmp(argv[1],"three"), idle=!strcmp(argv[1],"idle");
    bool overlap=!strcmp(argv[1],"overlap");
    assert(one||two||three||idle||overlap);
    PGRAPHVkState r={.num_active_frames=one?1:two?2:3,.bitmap_size=3};
    PGRAPHState pg={.vk_renderer_state=&r};
    memset(buffers,0xa5,sizeof(buffers));
    for(int i=0;i<3;i++) r.frame_staging[i]=(FrameStagingState){
        .vertex_ram={buffers[i],RAM_BYTES},.vertex_ram_flush_min=VK_WHOLE_SIZE,
        .vertex_ram_propagate_min=VK_WHOLE_SIZE,.uploaded_bitmap=&bitmaps[i]};
    pgraph_vk_update_vertex_ram_buffer(&pg,0,expected,RAM_BYTES);
    r.frame_staging[0].vertex_ram_initialized=true;
    for(int i=0;i<r.num_active_frames;i++) assert(rotate(&pg));
    bool passed=true;
    for(unsigned i=0;i<6;i++) {
        update(&pg,overlap?0:(i%3)*TARGET_PAGE_SIZE,(uint8_t)(i+1));
        if(!rotate(&pg)) { passed=false; if(!idle) break; }
    }
    if(idle) {
        for(int i=0;i<6;i++) if(!rotate(&pg)) passed=false;
        copied=0;
        for(int i=0;i<6;i++) if(!rotate(&pg)) passed=false;
        passed &= copied==0; /* Do not carry historical writes forever. */
    }
    printf("original frame RAM %s: %s\n",argv[1],passed?"PASS":"FAIL");
    return passed?0:1;
}
