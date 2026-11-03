/* SPDX-License-Identifier: MIT
 * Original metadata-only contract. No target, game or debugger is contacted.
 */
#include "exec/gdbstub.h"
#include <string.h>
extern const GDBFeature gdb_static_features[];
int main(void) {
    for (const GDBFeature *f = gdb_static_features; f->xmlname; ++f) {
        if (!strcmp(f->xmlname, "i386-32bit.xml")) {
            return !(f->num_regs > 8 && !strcmp(f->regs[2], "edx") &&
                     !strcmp(f->regs[8], "eip"));
        }
    }
    return 1;
}
