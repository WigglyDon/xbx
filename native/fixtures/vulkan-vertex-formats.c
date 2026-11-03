/* SPDX-License-Identifier: MIT
 * Original format-capability check. Reports only public format support and
 * geometry-shader booleans; no device names, identifiers, content or settings.
 */
#include <stdio.h>
#include <vulkan/vulkan.h>

int main(void)
{
    VkApplicationInfo app = {.sType=VK_STRUCTURE_TYPE_APPLICATION_INFO,
        .pApplicationName="xbx original vertex format check",
        .apiVersion=VK_API_VERSION_1_0};
    VkInstanceCreateInfo info = {.sType=VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        .pApplicationInfo=&app};
    VkInstance instance;
    if (vkCreateInstance(&info, NULL, &instance) != VK_SUCCESS) return 1;
    uint32_t count = 0;
    if (vkEnumeratePhysicalDevices(instance, &count, NULL) != VK_SUCCESS || count != 1) {
        vkDestroyInstance(instance, NULL);
        return 2; /* Do not guess which device the product selected. */
    }
    VkPhysicalDevice gpu;
    if (vkEnumeratePhysicalDevices(instance, &count, &gpu) != VK_SUCCESS) {
        vkDestroyInstance(instance, NULL);
        return 3;
    }
    const struct { const char *name; VkFormat format; } formats[] = {
#define FORMAT(x) {#x, VK_FORMAT_##x}
        FORMAT(R8_UNORM), FORMAT(R8G8_UNORM), FORMAT(R8G8B8_UNORM), FORMAT(R8G8B8A8_UNORM),
        FORMAT(R16_SNORM), FORMAT(R16G16_SNORM), FORMAT(R16G16B16_SNORM), FORMAT(R16G16B16A16_SNORM),
        FORMAT(R16_SSCALED), FORMAT(R16G16_SSCALED), FORMAT(R16G16B16_SSCALED), FORMAT(R16G16B16A16_SSCALED),
        FORMAT(R16_SINT), FORMAT(R16G16_SINT), FORMAT(R16G16B16_SINT), FORMAT(R16G16B16A16_SINT),
        FORMAT(R32_SFLOAT), FORMAT(R32G32_SFLOAT), FORMAT(R32G32B32_SFLOAT), FORMAT(R32G32B32A32_SFLOAT),
        FORMAT(R32_SINT),
#undef FORMAT
    };
    printf("{\"vertex_buffer\":{");
    for (unsigned i=0; i<sizeof(formats)/sizeof(formats[0]); i++) {
        VkFormatProperties props;
        vkGetPhysicalDeviceFormatProperties(gpu, formats[i].format, &props);
        printf("%s\"%s\":%s", i ? "," : "", formats[i].name,
            props.bufferFeatures & VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT ? "true" : "false");
    }
    VkPhysicalDeviceFeatures features;
    vkGetPhysicalDeviceFeatures(gpu, &features);
    printf("},\"geometry_shader\":%s,\"geometry_point_size\":%s}\n",
        features.geometryShader ? "true" : "false",
        features.shaderTessellationAndGeometryPointSize ? "true" : "false");
    vkDestroyInstance(instance, NULL);
    return 0;
}
