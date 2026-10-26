/* SPDX-License-Identifier: MIT
 * Original host contract for the actual patched Android monitor table and
 * comparator. These stand-ins test routing only, never a VM or serializer.
 */
#define _GNU_SOURCE
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

typedef struct Monitor { int unused; } Monitor;
typedef struct QDict { int unused; } QDict;
typedef struct Error Error;
typedef struct HumanReadableText HumanReadableText;
typedef struct HMPCommand HMPCommand;
struct HMPCommand {
    const char *name;
    const char *args_type;
    const char *params;
    const char *help;
    void (*cmd)(Monitor *, const QDict *);
    HMPCommand *sub_table;
    const char *flags;
};
static unsigned saved, loaded, listed;
static void hmp_savevm(Monitor *mon, const QDict *dict)
{ (void)mon; (void)dict; saved++; }
static void hmp_loadvm(Monitor *mon, const QDict *dict)
{ (void)mon; (void)dict; loaded++; }
static void hmp_info_snapshots(Monitor *mon, const QDict *dict)
{ (void)mon; (void)dict; listed++; }
#define qemu_strchrnul strchrnul
#include SUBJECT

#define CHECK(test, code) do { if (!(test)) { \
    fprintf(stderr, "monitor routing case %d failed\n", code); return code; \
} } while (0)
static HMPCommand *find(HMPCommand *commands, const char *name)
{
    for (HMPCommand *command = commands; command->name; command++) {
        if (hmp_compare_cmd(name, command->name)) return command;
    }
    return NULL;
}
int main(void)
{
    CHECK(hmp_compare_cmd("savevm", "savevm") == 1, 31);
    CHECK(hmp_compare_cmd("loadvm", "savevm") == 0, 32);
    CHECK(hmp_compare_cmd("loadvm", "savevm|loadvm") == 1, 33);
    CHECK(hmp_compare_cmd("save", "savevm") == 0, 34);
    CHECK(hmp_compare_cmd("savevm-extra", "savevm") == 0, 35);
    HMPCommand *save = find(hmp_cmds, "savevm");
    HMPCommand *load = find(hmp_cmds, "loadvm");
    HMPCommand *info = find(hmp_cmds, "info");
    CHECK(save && load && info && info->sub_table, 36);
    CHECK(save->cmd == hmp_savevm && load->cmd == hmp_loadvm, 37);
    CHECK(!strcmp(save->args_type, "name:s?") &&
          !strcmp(load->args_type, "name:s"), 38);
    CHECK(find(hmp_cmds, "loadvm-extra") == NULL &&
          find(hmp_cmds, "unknown") == NULL, 39);
    HMPCommand *snapshots = find(info->sub_table, "snapshots");
    CHECK(snapshots && snapshots->cmd == hmp_info_snapshots, 40);
    save->cmd(NULL, NULL);
    CHECK(saved == 1 && loaded == 0 && listed == 0, 41);
    load->cmd(NULL, NULL);
    CHECK(saved == 1 && loaded == 1 && listed == 0, 42);
    snapshots->cmd(NULL, NULL);
    CHECK(saved == 1 && loaded == 1 && listed == 1, 43);
    puts("actual Android session command routing passed; no VM executed");
    return 0;
}
