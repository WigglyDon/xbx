/* SPDX-License-Identifier: MIT
 * Original delayed-presentation model executing the retained submit/present
 * block. Render fences signal immediately; only acquiring the same image
 * establishes consumption of that image's pending presentation wait.
 * No GPU, game data or device is used.
 */
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#define VK_STRUCTURE_TYPE_SUBMIT_INFO 1
#define VK_STRUCTURE_TYPE_PRESENT_INFO_KHR 2
#define VK_PIPELINE_STAGE_ALL_COMMANDS_BIT 1
#define VK_CHECK(x) assert((x)==0)
typedef unsigned VkPipelineStageFlags;
typedef struct { int sType; unsigned commandBufferCount; int *pCommandBuffers;
 unsigned waitSemaphoreCount; int *pWaitSemaphores; unsigned *pWaitDstStageMask;
 unsigned signalSemaphoreCount; int *pSignalSemaphores; } VkSubmitInfo;
typedef struct { int sType; unsigned waitSemaphoreCount; int *pWaitSemaphores;
 unsigned swapchainCount; int *pSwapchains; unsigned *pImageIndices; } VkPresentInfoKHR;
typedef int VkResult;
typedef struct { bool direct_present; unsigned render_idx;
 int image_available[3],render_finished[3],present_fences[3],image_in_flight[3],swapchain;
} PGRAPHVkDisplayState;
typedef struct { int device,queue; PGRAPHVkDisplayState display; } PGRAPHVkState;
static int pending_image[3], violated, signaled;
static unsigned present_count, current_picture, displayed[4];
static int vkResetFences(int device,int count,int *fence)
{ assert(device==1 && count==1 && *fence>=10); return 0; }
static int vkQueueSubmit(int queue,int count,VkSubmitInfo *info,int fence)
{
 assert(queue==2 && count==1 && fence>=10 && info->signalSemaphoreCount==1);
 signaled=*info->pSignalSemaphores;
 assert(signaled>=0 && signaled<3);
 if(pending_image[signaled]>=0) violated=1;
 return 0;
}
static int vkQueuePresentKHR(int queue,VkPresentInfoKHR *info)
{
 assert(queue==2 && info->waitSemaphoreCount==1 && *info->pWaitSemaphores==signaled);
 pending_image[signaled]=(int)*info->pImageIndices;
 displayed[present_count++]=current_picture;
 return 0;
}
#include SUBJECT
int main(int argc,char **argv)
{
 assert(argc==2);unsigned pattern=0;assert(sscanf(argv[1],"%u",&pattern)==1 && pattern<81);
 PGRAPHVkState r={.device=1,.queue=2,.display={.direct_present=true,
 .image_available={20,21,22},.render_finished={0,1,2},.present_fences={10,11,12},.swapchain=7}};
 for(unsigned i=0;i<3;i++)pending_image[i]=-1;
 for(unsigned frame=0;frame<4;frame++) {
  unsigned image=pattern%3;pattern/=3;
  /* Model the acquire wait that precedes the captured submission block. */
  for(unsigned sem=0;sem<3;sem++)if(pending_image[sem]==(int)image)pending_image[sem]=-1;
  r.display.render_idx=image;current_picture=frame/2+1;
  subject(&r,frame%3);
 }
 assert(present_count==4 && displayed[0]==1 && displayed[1]==1 && displayed[2]==2 && displayed[3]==2);
 return violated?1:0;
}
