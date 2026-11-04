/* SPDX-License-Identifier: MIT
 * Original handle-lifetime controls for exact retained presentation branches.
 * No GPU, native window, game data or live device is accessed.
 */
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#define VK_NULL_HANDLE 0
#define VK_SUCCESS 0
#define VK_SUBOPTIMAL_KHR 1
#define VK_ERROR_OUT_OF_DATE_KHR -1
#define VK_ERROR_SURFACE_LOST_KHR -2
#define VK_ERROR_DEVICE_LOST -3
#define ANDROID_LOG_ERROR 1
#define ANDROID_LOG_WARN 2
#define __android_log_print(...) ((void)0)
typedef int VkInstance;
typedef int VkSurfaceKHR;
typedef int VkResult;
typedef struct { int unused; } PGRAPHVkDisplayState;
typedef struct { VkInstance instance; VkSurfaceKHR present_surface; PGRAPHVkDisplayState display; } PGRAPHVkState;
typedef struct { PGRAPHVkState *vk_renderer_state; } PGRAPHState;
static bool swapchain, window_ready;
static unsigned idle_waits, destroyed_surfaces, creates;
void destroy_current_display_image(PGRAPHState *pg)
{ (void)pg; if (swapchain) idle_waits++; swapchain=false; }
void vkDestroySurfaceKHR(VkInstance instance,VkSurfaceKHR surface,void *allocator)
{ assert(instance==1 && surface==7 && !allocator && !swapchain); destroyed_surfaces++; }
bool xemu_android_vulkan_create_surface(VkInstance instance,VkSurfaceKHR *surface)
{ assert(instance==1); creates++; if(!window_ready)return false; *surface=11; return true; }

#include SUBJECT

int main(int argc,char **argv)
{
    assert(argc==2);
    const char *name=argv[1];
    PGRAPHVkState r={.instance=1,.present_surface=7};
    PGRAPHState pg={.vk_renderer_state=&r};
    swapchain=true;
    bool acquire=strncmp(name,"acquire-",8)==0;
    bool present=strncmp(name,"present-",8)==0;
    bool caps=strncmp(name,"caps-",5)==0;
    bool retry=strcmp(name,"retry")==0;
    assert(acquire||present||caps||retry);
    bool lost=strstr(name,"-lost")!=NULL && strstr(name,"device")==NULL;
    bool outdated=strstr(name,"outdated")!=NULL;
    bool device=strstr(name,"device")!=NULL;
    bool suboptimal=strstr(name,"suboptimal")!=NULL;
    VkResult result=lost?VK_ERROR_SURFACE_LOST_KHR:outdated?VK_ERROR_OUT_OF_DATE_KHR:device?VK_ERROR_DEVICE_LOST:suboptimal?VK_SUBOPTIMAL_KHR:VK_SUCCESS;
    if(acquire)subject_acquire(&pg,result);
    if(present)subject_present(&pg,result);
    if(caps)(void)subject_caps(&pg,result);
    bool passed=true;
    if(retry) {
        subject_acquire(&pg,VK_ERROR_SURFACE_LOST_KHR);
        passed &= r.present_surface==0 && !swapchain && destroyed_surfaces==1 && idle_waits==1;
        passed &= !create_android_swapchain(&pg,640,480) && r.present_surface==0;
        window_ready=true;
        passed &= create_android_swapchain(&pg,640,480) && r.present_surface==11;
        unsigned before=creates;
        passed &= create_android_swapchain(&pg,640,480) && creates==before;
        passed &= destroyed_surfaces==1;
    } else if(lost) {
        passed &= r.present_surface==0 && !swapchain && destroyed_surfaces==1 && idle_waits==1;
    } else {
        passed &= r.present_surface==7 && destroyed_surfaces==0 && creates==0;
        passed &= swapchain==!outdated;
        passed &= idle_waits==(outdated?1u:0u);
    }
    printf("original presentation lifetime %s: %s\n",name,passed?"PASS":"FAIL");
    return passed?0:1;
}
