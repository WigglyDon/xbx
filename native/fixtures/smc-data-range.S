/* SPDX-License-Identifier: MIT
 * Original same-page data writes and genuine direct/physical-alias/cross-page/
 * current-block instruction changes. No game content or addresses are inputs.
 */
.text
.globl _start
_start:
    cld
    call serial_init
    call mutable_fn
    cmpl $1,%eax
    jne fail
    movl $ready,%esi
    call text
command:
    movw $0x3fd,%dx
    inb %dx,%al
    testb $1,%al
    jnz read_command
    sti
    hlt
    jmp command
read_command:
    movw $0x3f8,%dx
    inb %dx,%al
    cmpb $'c',%al
    je control
    cmpb $'d',%al
    je data_write
    cmpb $'m',%al
    je mutate
    jmp command
control:
    movl $1,stage
    movl $10000,%ecx
1:  call mutable_fn
    cmpl $1,%eax
    jne fail
    loop 1b
    movl $control_pass,%esi
    call text
    jmp command
data_write:
    movl $2,stage
    movl $10000,%ecx
2:  movl %ecx,scratch
    call mutable_fn
    cmpl $1,%eax
    jne fail
    loop 2b
    movl $data_pass,%esi
    call text
    jmp command
mutate:
    movl $3,stage
    movl $2,mutable_fn+1
    call mutable_fn
    cmpl $2,%eax
    jne fail
    movl $direct_pass,%esi
    call text
    movl $4,stage
    pushl $mutable_fn+1
    call *thunks                 /*173 MmGetPhysicalAddress */
    cmpl $0x4000000,%eax
    jae fail
    movl %eax,%edx
    andl $4095,%edx
    movl %edx,alias_offset
    andl $-4096,%eax
    pushl $4                    /* PAGE_READWRITE */
    pushl $4096
    pushl %eax
    call *thunks+4              /*177 MmMapIoSpace: alias our own data page */
    testl %eax,%eax
    jz fail
    movl %eax,alias_base
    movl alias_offset,%edx
    movl $6,(%eax,%edx)
    call mutable_fn
    cmpl $6,%eax
    jne fail
    pushl $4096
    pushl alias_base
    call *thunks+8              /*183 MmUnmapIoSpace */
    movl $alias_pass,%esi
    call text
    movl $5,stage
    call cross_page_fn
    cmpl $1,%eax
    jne fail
    movb $1,cross_page_tail
    call cross_page_fn
    cmpl $65537,%eax
    jne fail
    movb $0,cross_page_tail
    call cross_page_fn
    cmpl $1,%eax
    jne fail
    movl $cross_pass,%esi
    call text
    movl $6,stage
    call self_next
    cmpl $4,%eax
    jne fail
    call self_modify
    cmpl $5,%eax
    jne fail
    movl $current_pass,%esi
    call text
    movl $mutate_pass,%esi
    call text
    jmp command
fail:
    movl $failed,%esi
    call text
    movl stage,%eax
    call hex
    movl $newline,%esi
    call text
3:  sti
    hlt
    jmp 3b
/*__XBX_ORIGINAL_UART__*/
/* MOV begins three bytes before the code/data page boundary. */
.org 4093
cross_page_fn: .byte 0xb8,1,0
.data
cross_page_tail: .byte 0,0,0xc3
.balign 4
.globl thunks
thunks: .long 0x800000ad,0x800000b1,0x800000b7,0
.balign 16
mutable_fn: .byte 0xb8
    .long 1
    .byte 0xc3
.balign 16
self_modify:
    movl $5,self_next+1
self_next:
    movl $4,%eax
    ret
.balign 64
scratch: .long 0
stage: .long 0
alias_offset: .long 0
alias_base: .long 0
superio: .byte 7,4,0x60,3,0x61,0xf8,0x70,4,0x30,1
ready: .asciz "XBX SMC READY\n"
control_pass: .asciz "XBX SMC CONTROL PASS\n"
data_pass: .asciz "XBX SMC DATA PASS\n"
direct_pass: .asciz "XBX SMC DIRECT PASS\n"
alias_pass: .asciz "XBX SMC ALIAS PASS\n"
cross_pass: .asciz "XBX SMC CROSS_PAGE PASS\n"
current_pass: .asciz "XBX SMC CURRENT_BLOCK PASS\n"
mutate_pass: .asciz "XBX SMC MUTATE PASS\n"
failed: .asciz "XBX SMC FAIL stage="
newline: .asciz "\n"
