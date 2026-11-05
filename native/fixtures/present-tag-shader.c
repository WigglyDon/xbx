/* SPDX-License-Identifier: MIT
 * Compile the actual retained display shader and resource limits; no GPU or
 * commercial game is involved. SUBJECT keeps its LGPL-2.0-or-later provenance.
 */
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <glslang/Include/glslang_c_interface.h>
#include SUBJECT
int main(int argc, char **argv)
{
    if (argc != 2 || !glslang_initialize_process()) return 2;
    const glslang_input_t input = {
        .language = GLSLANG_SOURCE_GLSL,
        .stage = GLSLANG_STAGE_FRAGMENT,
        .client = GLSLANG_CLIENT_VULKAN,
        .client_version = GLSLANG_TARGET_VULKAN_1_1,
        .target_language = GLSLANG_TARGET_SPV,
        .target_language_version = GLSLANG_TARGET_SPV_1_3,
        .code = display_frag_glsl,
        .default_version = 460,
        .default_profile = GLSLANG_NO_PROFILE,
        .messages = GLSLANG_MSG_DEFAULT_BIT,
        .resource = &resource_limits,
    };
    glslang_shader_t *shader = glslang_shader_create(&input);
    if (!shader || !glslang_shader_preprocess(shader, &input) ||
        !glslang_shader_parse(shader, &input)) {
        if (shader) fputs(glslang_shader_get_info_log(shader), stderr);
        return 3;
    }
    glslang_program_t *program = glslang_program_create();
    glslang_program_add_shader(program, shader);
    if (!glslang_program_link(program, GLSLANG_MSG_SPV_RULES_BIT |
                                      GLSLANG_MSG_VULKAN_RULES_BIT)) {
        fputs(glslang_program_get_info_log(program), stderr);
        return 4;
    }
    glslang_program_SPIRV_generate(program, GLSLANG_STAGE_FRAGMENT);
    size_t count = glslang_program_SPIRV_get_size(program);
    if (!count) return 5;
    unsigned int *words = malloc(count * sizeof(*words));
    if (!words) return 6;
    glslang_program_SPIRV_get(program, words);
    FILE *out = fopen(argv[1], "wb");
    if (!out || fwrite(words, sizeof(*words), count, out) != count ||
        fclose(out)) return 7;
    free(words);
    glslang_program_delete(program);
    glslang_shader_delete(shader);
    glslang_finalize_process();
    return 0;
}
