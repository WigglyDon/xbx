/* SPDX-License-Identifier: MIT
 * Original demand-commit control for the public user-buffer I/O boundary.
 * Reuses the exception-registration ABI of nullcall.S. It handles only faults
 * inside this fixture's reserved header and four-page buffer, commits the faulting page,
 * and retries the unchanged instruction. No game code or data.
 */
.text
demand_handler:
    pushl %ebp
    movl %esp,%ebp
    subl $8,%esp
    pushl %ebx
    pushl %esi
    pushl %edi
    movl 8(%ebp),%eax           /* EXCEPTION_RECORD */
    cmpl $0xc0000005,(%eax)
    jne demand_search
    testl $6,4(%eax)            /* Never handle unwind/exit notifications. */
    jnz demand_search
    cmpl $2,16(%eax)
    jb demand_search
    movl 24(%eax),%edx          /* Fault address, not the instruction address. */
    cmpl $buffer-4096,%edx
    jb demand_search
    cmpl $buffer+16384,%edx
    jae demand_search
    cmpl $5,demand_faults
    jae demand_search
    andl $0xfffff000,%edx
    movl %edx,-8(%ebp)
    movl $4096,-4(%ebp)
    call *thunks+44             /* KeGetCurrentIrql: commitment must be pageable. */
    cmpb $1,%al
    ja demand_search
    pushl $4                   /* PAGE_READWRITE */
    pushl $0x1000              /* MEM_COMMIT inside the existing reservation */
    leal -4(%ebp),%eax
    pushl %eax
    pushl $0
    leal -8(%ebp),%eax
    pushl %eax
    call *thunks+36
    testl %eax,%eax
    js demand_search
    incl demand_faults
    xorl %eax,%eax             /* ExceptionContinueExecution */
    jmp demand_return
demand_search:
    movl $1,%eax               /* ExceptionContinueSearch */
demand_return:
    popl %edi
    popl %esi
    popl %ebx
    leave
    ret
