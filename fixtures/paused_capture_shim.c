/* SPDX-License-Identifier: MIT
 * Mechanical host adapter for extracted GPL-lineage public HMP function.
 * Rust supplies original lifecycle/owner/error controls. This is not an
 * emulated-CPU ABI or a replacement device serializer.
 */
#include <stdbool.h>
#include <stddef.h>
typedef struct Monitor Monitor;
typedef struct QDict QDict;
typedef struct Error Error;
typedef int RunState;
enum { RUN_STATE_RUNNING=1, RUN_STATE_PAUSED=2, RUN_STATE_SAVE_VM=3 };
extern RunState runstate_get(void);
extern bool runstate_is_running(void);
extern void vm_start(void);
extern int vm_stop(RunState state);
extern bool save_snapshot(const char*,bool,const char*,bool,void*,Error**);
extern void hmp_handle_error(Monitor*,Error*);
static const char *qdict_get_try_str(const QDict *d,const char *key)
{ (void)d;(void)key;return "original"; }
#include "subject.inc"
void original_capture(void) { hmp_savevm(NULL,NULL); }
