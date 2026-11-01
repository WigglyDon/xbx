/* SPDX-License-Identifier: MIT
 * Original controlled IR contract for unchanged retained optimizer functions.
 * A modeled fault observes architectural globals before the next instruction.
 * This is not a vCPU, Android ABI, exception delivery or commercial-game test.
 */
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "qemu/queue.h"
typedef uintptr_t TCGArg;
typedef struct TCGTemp { const char *name; uint32_t value; } TCGTemp;
typedef enum { INDEX_op_mov, INDEX_op_set_label, INDEX_op_call, INDEX_op_insn_start,
       INDEX_op_goto_tb, INDEX_op_load, INDEX_op_exit } TCGOpcode;
enum { TCG_OPF_BB_EXIT=1, TCG_OPF_BB_END=2, TCG_OPF_CALL_CLOBBER=4,
       TCG_OPF_SIDE_EFFECTS=8, TCG_OPF_COND_BRANCH=128 };
typedef struct TCGOpDef { int nb_oargs, nb_iargs, flags; } TCGOpDef;
static const TCGOpDef tcg_op_defs[] = {
    [INDEX_op_mov]={1,1,0}, [INDEX_op_load]={1,1,TCG_OPF_SIDE_EFFECTS|TCG_OPF_CALL_CLOBBER},
    [INDEX_op_goto_tb]={0,0,TCG_OPF_BB_END}, [INDEX_op_exit]={0,0,TCG_OPF_BB_EXIT}
};
typedef struct TCGOp {
    int opc; TCGArg args[4]; bool removed;
    QTAILQ_ENTRY(TCGOp) link;
} TCGOp;
typedef struct TCGContext {
    TCGTemp temps[8]; int nb_globals; uint8_t succ_cc_defines[2];
    QTAILQ_HEAD(, TCGOp) ops;
} TCGContext;
static TCGTemp *arg_temp(TCGArg value) { return (TCGTemp *)value; }
static void tcg_op_remove(TCGContext *s, TCGOp *op)
{ op->removed=true; QTAILQ_REMOVE(&s->ops,op,link); }
#define XBOX 1
#include SUBJECT

static void init(TCGContext *s)
{
    memset(s,0,sizeof(*s)); QTAILQ_INIT(&s->ops); s->nb_globals=4;
    const char *names[]={"cc_op","cc_dst","cc_src","cc_src2"};
    for(int i=0;i<4;i++) s->temps[i]=(TCGTemp){names[i],0xbad};
    s->temps[4].value=0x3456; s->temps[5].value=0x5678;
}
static void add(TCGContext *s,TCGOp *op,int code,TCGTemp *out,TCGTemp *in)
{
    *op=(TCGOp){.opc=code,.args={(TCGArg)out,(TCGArg)in}};
    QTAILQ_INSERT_TAIL(&s->ops,op,link);
}
static uint32_t observe(TCGContext *s)
{
    TCGOp *op;
    QTAILQ_FOREACH(op,&s->ops,link) {
        if(op->opc==INDEX_op_load || op->opc==INDEX_op_goto_tb) break;
        if(op->opc==INDEX_op_mov) arg_temp(op->args[0])->value=arg_temp(op->args[1])->value;
    }
    return s->temps[1].value;
}
int main(int argc,char **argv)
{
    assert(argc==2);TCGContext s;TCGOp op[4];init(&s);
    bool cross=!strncmp(argv[1],"cross-",6);
    bool fault=strstr(argv[1],"fault")!=NULL;
    bool pass;
    if(cross) {
        add(&s,&op[0],fault?INDEX_op_load:INDEX_op_mov,fault?&s.temps[6]:&s.temps[1],&s.temps[4]);
        add(&s,&op[1],fault?INDEX_op_mov:INDEX_op_load,fault?&s.temps[1]:&s.temps[6],&s.temps[5]);
        add(&s,&op[2],INDEX_op_exit,NULL,NULL);
        uint8_t mask=tier1_compute_cc_defines_first(&s);
        init(&s);s.succ_cc_defines[0]=mask;
        add(&s,&op[0],INDEX_op_mov,&s.temps[1],&s.temps[4]);
        add(&s,&op[1],INDEX_op_goto_tb,NULL,NULL);
        tier1_cross_tb_dead_flag_elimination(&s);
        pass=fault ? observe(&s)==0x3456 && !op[0].removed : op[0].removed && (mask&2);
    } else {
        add(&s,&op[0],INDEX_op_mov,&s.temps[1],&s.temps[4]);
        add(&s,&op[1],fault?INDEX_op_load:INDEX_op_insn_start,&s.temps[6],&s.temps[4]);
        add(&s,&op[2],INDEX_op_mov,&s.temps[1],&s.temps[5]);
        add(&s,&op[3],INDEX_op_exit,NULL,NULL);
        tier1_dead_flag_elimination(&s);
        pass=fault ? observe(&s)==0x3456 && !op[0].removed : observe(&s)==0x5678 && op[0].removed;
    }
    printf("%s: %s (controlled IR; no guest executed)\n",argv[1],pass?"PASS":"FAIL");
    return pass?0:1;
}
