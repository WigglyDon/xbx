/* SPDX-License-Identifier: MIT
 * Original finite public-platform contract. Parent entry returns normally in
 * EXIT case; the worker must successfully wait on an independently opened
 * parent-thread handle before reporting completion. Context is image-owned,
 * never on the departed stack. CONTROL keeps its parent alive until child exit.
 * Public interfaces: nxdk a6c31ee90540710a3df26a0712b61492f2ea03dd.
 * Ordinary guest UART only; no host title-memory observer or private code.
 */
.text
.globl _start
_start:
    pushl %ebp
    pushl %edi
    pushl %esi
    pushl %ebx
    cld
    call serial_init
    movl $boot,%esi
    call text
    movl $1,stage
    call *thunks                 /* KeGetCurrentThread */
    testl %eax,%eax
    jz fail
    movl %eax,parent_thread
    pushl $parent_handle
    pushl thunks+4               /* PsThreadObjectType data address */
    pushl %eax
    call *thunks+8               /* ObOpenObjectByPointer */
    movl %eax,status
    testl %eax,%eax
    jnz fail
    cmpl $0,parent_handle
    je fail
    movl $2,stage
    pushl $system_start
    pushl $0                     /* debugger */
    pushl $0                     /* initially runnable */
    pushl $context
    pushl $work
    pushl $child_id
    pushl $0                     /* TLS bytes */
    pushl $0x4000
    pushl $0                     /* extension bytes */
    pushl $child_handle
    call *thunks+12              /* PsCreateSystemThreadEx */
    movl %eax,status
    testl %eax,%eax
    jnz fail
#if LIFETIME_EXIT
    movl $0x72657431,parent_returning
    popl %ebx
    popl %esi
    popl %edi
    popl %ebp
    ret                          /* existing loader ends this initial thread */
#else
    movl $3,stage
    pushl $timeout
    pushl $0
    pushl child_handle
    call *thunks+16              /* finite wait for child termination */
    movl %eax,status
    testl %eax,%eax
    jnz fail
    cmpl $0x600dcafe,child_done
    jne fail
    pushl child_handle
    call *thunks+20              /* NtClose */
    movl %eax,status
    testl %eax,%eax
    jnz fail
    movl $control_complete,%esi
    call text
    jmp done
#endif
system_start:
    movl $4,stage
    cmpl $work,4(%esp)
    jne fail
    cmpl $context,8(%esp)
    jne fail
    pushl 8(%esp)
    call *8(%esp)
    pushl %eax
    call *thunks+24              /* PsTerminateSystemThread; never returns */
    movl $5,stage
    jmp fail
work:
    pushl %ebp
    pushl %edi
    pushl %esi
    pushl %ebx
    movl $6,stage
    cmpl $context,20(%esp)
    jne fail
    cmpl $0x58424331,context
    jne fail
    cmpl $0xc0ffee91,context+4
    jne fail
    cmpl $0x13579bdf,context+8
    jne fail
    cmpl $0x2468ace0,context+12
    jne fail
    call *thunks
    cmpl parent_thread,%eax
    je fail
#if LIFETIME_EXIT
    movl $7,stage
    pushl $timeout
    pushl $0
    pushl parent_handle
    call *thunks+16              /* success proves parent is now terminated */
    movl %eax,status
    testl %eax,%eax
    jnz fail
    cmpl $0x72657431,parent_returning
    jne fail
    /* Recheck persistent input after the parent stack has been reaped. */
    cmpl $0xc0ffee91,context+4
    jne fail
    cmpl $0x2468ace0,context+12
    jne fail
#endif
    movl $8,stage
    pushl parent_handle
    call *thunks+20
    movl %eax,status
    testl %eax,%eax
    jnz fail
#if LIFETIME_EXIT
    pushl child_handle
    call *thunks+20
    movl %eax,status
    testl %eax,%eax
    jnz fail
    movl $exit_complete,%esi
#else
    movl $control_child,%esi
#endif
    movl $0x600dcafe,child_done
    call text
    xorl %eax,%eax
    popl %ebx
    popl %esi
    popl %edi
    popl %ebp
    ret $4
fail:
    movl $failed,%esi
    call text
    movl stage,%eax
    call hex
    movl $status_text,%esi
    call text
    movl status,%eax
    call hex
    movl $newline,%esi
    call text
done:
    sti
    hlt
    jmp done
serial_init:
    movw $0x2e,%dx
    movb $0x55,%al
    outb %al,%dx
    movl $superio,%esi
    movl $5,%ecx
1:  lodsb
    movw $0x2e,%dx
    outb %al,%dx
    lodsb
    incw %dx
    outb %al,%dx
    loop 1b
    movw $0x2e,%dx
    movb $0xaa,%al
    outb %al,%dx
    movw $0x3fb,%dx
    movb $0x80,%al
    outb %al,%dx
    movw $0x3f8,%dx
    movb $1,%al
    outb %al,%dx
    incw %dx
    xorb %al,%al
    outb %al,%dx
    movw $0x3fb,%dx
    movb $3,%al
    outb %al,%dx
    ret
text:
    lodsb
    testb %al,%al
    jz 2f
    call character
    jmp text
2:  ret
character:
    pushl %eax
    pushl %ecx
    movl $1000000,%ecx
    movw $0x3fd,%dx
3:  inb %dx,%al
    testb $0x20,%al
    jnz 4f
    loop 3b
4:  popl %ecx
    popl %eax
    movw $0x3f8,%dx
    outb %al,%dx
    ret
hex:
    pushl %ebx
    pushl %ecx
    movl %eax,%ebx
    movl $8,%ecx
5:  roll $4,%ebx
    movl %ebx,%eax
    andb $15,%al
    addb $'0',%al
    cmpb $'9',%al
    jbe 6f
    addb $7,%al
6:  call character
    loop 5b
    popl %ecx
    popl %ebx
    ret
.balign 4
.globl thunks
thunks:
    .long 0x80000068             /*104 KeGetCurrentThread */
    .long 0x80000103             /*259 PsThreadObjectType data */
    .long 0x800000f4             /*244 ObOpenObjectByPointer */
    .long 0x800000ff             /*255 PsCreateSystemThreadEx */
    .long 0x800000e9             /*233 NtWaitForSingleObject */
    .long 0x800000bb             /*187 NtClose */
    .long 0x80000102             /*258 PsTerminateSystemThread */
    .long 0
timeout: .quad -100000000
context: .long 0x58424331,0xc0ffee91,0x13579bdf,0x2468ace0
parent_thread: .long 0
parent_handle: .long 0
parent_returning: .long 0
child_handle: .long 0
child_id: .long 0
child_done: .long 0
stage: .long 0
status: .long 0
superio: .byte 7,4,0x60,3,0x61,0xf8,0x70,4,0x30,1
#if LIFETIME_EXIT
boot: .asciz "LIFETIME EXIT BOOT\n"
#else
boot: .asciz "LIFETIME CONTROL BOOT\n"
#endif
control_child: .asciz "LIFETIME CONTROL CHILD PASS\n"
control_complete: .asciz "LIFETIME CONTROL COMPLETE\n"
exit_complete: .asciz "LIFETIME EXIT AFTER_PARENT_TERMINATION COMPLETE\n"
failed: .asciz "LIFETIME FAIL stage="
status_text: .asciz " status="
newline: .asciz "\n"
