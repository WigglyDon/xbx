/* SPDX-License-Identifier: MIT
 * Original inputs exercising the retained renderer's exact binding/conversion
 * bodies. No Vulkan submission, guest/title data, or captured shaders.
 */
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <vulkan/vulkan_core.h>
#include "nv2a_regs.h"

#define NV2A_VERTEXSHADER_ATTRIBUTES 16
#define ARRAY_SIZE(a) (sizeof(a) / sizeof((a)[0]))
#define MAX(a,b) ((a) > (b) ? (a) : (b))
#define NV2A_VK_DGROUP_BEGIN(...) ((void)0)
#define NV2A_VK_DGROUP_END() ((void)0)
#define NV2A_VK_DPRINTF(...) ((void)0)
#define OPT_STAT_INC(...) ((void)0)
#define nv2a_profile_inc_counter(...) ((void)0)
typedef uint64_t hwaddr;
typedef struct {
    bool dma_select;
    hwaddr offset;
    unsigned inline_array_offset;
    float inline_value[4];
    unsigned format, size, count;
    uint32_t stride;
} VertexAttribute;
typedef struct { hwaddr addr, size; } MemorySyncRequirement;
typedef struct {
    uint32_t last_vertex_attr_gen;
    int cached_num_active_bindings, cached_num_active_attrs;
    struct { hwaddr base_addr; size_t stride; } cached_attr_layout[16];
    VkVertexInputAttributeDescription cached_attr_descs[16];
    VkVertexInputBindingDescription cached_bind_descs[16];
    int cached_attr_to_desc_loc[16];
    hwaddr cached_attr_offsets[16];
    uint32_t cached_compressed_attrs, cached_uniform_attrs;
    uint32_t cached_swizzle_attrs, cached_int16_attrs;
    bool sscaled_vertex_supported;
    int num_active_vertex_attribute_descriptions;
    int num_active_vertex_binding_descriptions;
    VkVertexInputAttributeDescription vertex_attribute_descriptions[16];
    VkVertexInputBindingDescription vertex_binding_descriptions[16];
    int vertex_attribute_to_description_location[16];
    hwaddr vertex_attribute_offsets[16];
    MemorySyncRequirement vertex_ram_buffer_syncs[16];
    unsigned num_vertex_ram_buffer_syncs;
} PGRAPHVkState;
typedef struct {
    PGRAPHVkState *vk_renderer_state;
    uint32_t vertex_attr_gen;
    uint32_t compressed_attrs, uniform_attrs, swizzle_attrs, int16_attrs;
    VertexAttribute vertex_attributes[16];
    uint32_t dma_vertex_a, dma_vertex_b;
    float inline_array[64];
} PGRAPHState;
typedef struct { PGRAPHState pgraph; uint8_t *vram_ptr; } NV2AState;
static unsigned mapping_calls;
static uint8_t *nv_dma_map(NV2AState *d, uint32_t object, hwaddr *length)
{
    (void)object;
    mapping_calls++;
    *length = 4096;
    return d->vram_ptr;
}

#include SUBJECT

static void values(uint8_t *ram, unsigned offset, float base)
{
    float v[4] = {base, base+1, base+2, base+3};
    memcpy(ram+offset, v, sizeof(v));
}

int main(int argc, char **argv)
{
    assert(argc == 2);
    bool control = !strcmp(argv[1], "control");
    bool uniform = !strcmp(argv[1], "uniform");
    bool provoking = !strcmp(argv[1], "provoking");
    bool disabled = !strcmp(argv[1], "disabled");
    assert(control || uniform || provoking || disabled);
    _Alignas(16) uint8_t ram[4096] = {0};
    PGRAPHVkState vk = {.sscaled_vertex_supported = true};
    NV2AState d = {.pgraph = {.vk_renderer_state = &vk, .vertex_attr_gen = 1},
                   .vram_ptr = ram};
    VertexAttribute *a = d.pgraph.vertex_attributes;
    a[0] = (VertexAttribute){.format=NV097_SET_VERTEX_DATA_ARRAY_FORMAT_TYPE_F,
        .size=4, .count=4, .stride=16, .offset=256};
    /* A valid constant array at byte zero must not be confused with a
     * disabled attribute whose cached address also happens to be zero. */
    a[1] = (VertexAttribute){.format=NV097_SET_VERTEX_DATA_ARRAY_FORMAT_TYPE_F,
        .size=4, .count=4, .stride=0, .offset=0};
    a[2].inline_value[0] = 91.0f;
    a[2].inline_value[3] = 1.0f;
    values(ram, 256, 10.0f);
    values(ram, 272, 20.0f);
    values(ram, 288, 30.0f);
    values(ram, 0, 40.0f);
    pgraph_vk_bind_vertex_attributes(&d, 0, 0, false, 0, 0);
    assert(mapping_calls == 2 && vk.cached_num_active_bindings == 1);
    assert(a[0].inline_value[0] == 10.0f && a[1].inline_value[0] == 40.0f);
    if (uniform || control) values(ram, 0, 50.0f);
    vk.num_vertex_ram_buffer_syncs = 0;
    if (control) d.pgraph.vertex_attr_gen++;
    pgraph_vk_bind_vertex_attributes(&d, provoking ? 1 : 0,
        provoking ? 2 : 0, false, 0, provoking ? 2 : 0);
    assert(mapping_calls == (control ? 4u : 2u));
    bool passed = a[0].inline_value[0] == (provoking ? 30.0f : 10.0f)
        && a[1].inline_value[0] == ((uniform || control) ? 50.0f : 40.0f)
        && a[2].inline_value[0] == 91.0f && a[2].inline_value[3] == 1.0f;
    printf("original vertex binding %s: %s\n", argv[1], passed ? "PASS" : "FAIL");
    return passed ? 0 : 1;
}
