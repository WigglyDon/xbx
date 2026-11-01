/* SPDX-License-Identifier: MIT
 * Original bounded native device-state fixture. Included only in the private
 * qualification build; never reads guest memory or draws a commercial frame.
 */
#include "system/runstate.h"

static uint32_t xbx_vertex_word(unsigned int vertex, unsigned int attr,
                                unsigned int component)
{
    return 0x3f000000 + (vertex << 18) + (attr << 12) + (component << 8);
}

static uint32_t xbx_vertex_format(unsigned int attr)
{
    unsigned int type = NV097_SET_VERTEX_DATA_ARRAY_FORMAT_TYPE_F;
    unsigned int count = 4;
    if (attr == 1) {
        type = NV097_SET_VERTEX_DATA_ARRAY_FORMAT_TYPE_UB_D3D;
    } else if (attr == 2) {
        type = NV097_SET_VERTEX_DATA_ARRAY_FORMAT_TYPE_CMP;
        count = 1;
    } else if (attr == 3) {
        type = NV097_SET_VERTEX_DATA_ARRAY_FORMAT_TYPE_S1;
        count = 2;
    }
    return type | (count << 4) | ((attr + 1) << 8);
}

static void xbx_vertex_method(NV2AState *d, unsigned int method, uint32_t word)
{
    pgraph_method(d, 0, method, word, &word, 1, 1, false);
}

char *nv2a_xbx_vertex_fixture(const char *action);
char *nv2a_xbx_vertex_fixture(const char *action)
{
    if (g_strcmp0(g_getenv("XBX_ORIGINAL_VERTEX_FIXTURE"), "1") ||
        runstate_is_running() || !g_nv2a ||
        (strcmp(action, "seed") && strcmp(action, "check"))) {
        return g_strdup("{\"refused\":true}");
    }
    NV2AState *d = g_nv2a;
    PGRAPHState *pg = &d->pgraph;
    nv2a_lock_fifo(d);
    if (!strcmp(action, "seed")) {
        pgraph_reg_w(pg, NV_PGRAPH_CTX_CONTROL, NV_PGRAPH_CTX_CONTROL_CHID);
        pgraph_reg_w(pg, NV_PGRAPH_CTX_CACHE1, 0x97);
        pgraph_reset_inline_buffers(pg);
        pg->primitive_mode = NV097_SET_BEGIN_END_OP_POINTS;
        for (unsigned int i = 0; i < NV2A_VERTEXSHADER_ATTRIBUTES; i++) {
            xbx_vertex_method(d, NV097_SET_VERTEX_DATA_ARRAY_FORMAT + i * 4,
                              xbx_vertex_format(i));
            xbx_vertex_method(d, NV097_SET_VERTEX_DATA_ARRAY_OFFSET + i * 4,
                              ((i & 1) << 31) | ((i + 1) * 64));
        }
        /* Position is last, completing each vertex after the other constants. */
        for (unsigned int v = 0; v < 3; v++) {
            for (int i = NV2A_VERTEXSHADER_ATTRIBUTES - 1; i >= 0; i--) {
                for (unsigned int c = 0; c < 4; c++) {
                    xbx_vertex_method(d, NV097_SET_VERTEX_DATA4F_M + i * 16 + c * 4,
                                      xbx_vertex_word(v, i, c));
                }
            }
        }
    }
    bool passed = pg->inline_buffer_length == 3 && pg->compressed_attrs == 4;
    unsigned int width = 0;
    unsigned int populated = 0;
    for (unsigned int i = 0; i < NV2A_VERTEXSHADER_ATTRIBUTES; i++) {
        VertexAttribute *a = &pg->vertex_attributes[i];
        uint32_t format = xbx_vertex_format(i);
        unsigned int size = i == 1 ? 1 : i == 3 ? 2 : 4;
        passed &= a->format == (format & 0xf) &&
                  a->count == ((format >> 4) & 0xf) && a->size == size &&
                  a->stride == i + 1 && a->dma_select == (i & 1) &&
                  a->offset == (i + 1) * 64 && a->inline_buffer_populated;
        width = ROUND_UP(width, size) + a->size * a->count;
        populated += a->inline_buffer_populated;
        for (unsigned int c = 0; c < 4; c++) {
            uint32_t bits;
            memcpy(&bits, &a->inline_value[c], sizeof(bits));
            passed &= bits == xbx_vertex_word(2, i, c);
            if (a->inline_buffer_populated && pg->inline_buffer_length == 3) {
                for (unsigned int v = 0; v < 3; v++) {
                    memcpy(&bits, &a->inline_buffer[v * 4 + c], sizeof(bits));
                    passed &= bits == xbx_vertex_word(v, i, c);
                }
            }
        }
    }
    unsigned int vertices = pg->inline_buffer_length;
    nv2a_unlock_fifo(d);
    return g_strdup_printf("{\"passed\":%s,\"vertex_bytes\":%u,"
                           "\"populated\":%u,\"vertices\":%u}",
                           passed ? "true" : "false", width, populated, vertices);
}
