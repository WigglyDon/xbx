/* SPDX-License-Identifier: MIT
 * Output-only adapter for the retained original npx-ownership.S regression.
 * Uses the same public UART activation as startup-lifetime.S. No observer.
 * All registers and flags are preserved; the regression and its data are reused.
 */
.text
npx_serial_init:
    pushfl
    pushal
    movw $0x2e,%dx
    movb $0x55,%al
    outb %al,%dx
    movl $npx_superio,%esi
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
    movl $npx_boot,%esi
    call npx_text
    popal
    popfl
    ret
npx_report:
    pushfl
    pushal
    movl %ecx,%ebx
    movl $npx_prefix,%esi
    call npx_text
    movl $8,%ecx
2:  roll $4,%ebx
    movl %ebx,%eax
    andb $15,%al
    addb $'0',%al
    cmpb $'9',%al
    jbe 3f
    addb $7,%al
3:  call npx_character
    loop 2b
    movb $10,%al
    call npx_character
    popal
    popfl
    ret
npx_text:
    lodsb
    testb %al,%al
    jz 4f
    call npx_character
    jmp npx_text
4:  ret
npx_character:
    pushl %eax
    pushl %ecx
    movl $1000000,%ecx
    movw $0x3fd,%dx
5:  inb %dx,%al
    testb $0x20,%al
    jnz 6f
    loop 5b
6:  popl %ecx
    popl %eax
    movw $0x3f8,%dx
    outb %al,%dx
    ret
npx_superio: .byte 7,4,0x60,3,0x61,0xf8,0x70,4,0x30,1
npx_boot: .asciz "NPX OWNERSHIP BOOT\n"
npx_prefix: .asciz "NPX RESULT="
