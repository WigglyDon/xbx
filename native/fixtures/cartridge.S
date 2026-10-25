/* SPDX-License-Identifier: MIT
 * Original synthetic cartridge. No SDK/game implementation. Fixed COM1 terminal
 * commands are this program's input; no host guest-memory reads. ATA sector1000
 * is outside the generated FATX partitions and is used ONLY in synthetic disks.
 * RAM state/cookie and the separately written durable marker distinguish resume
 * from cold boot. This does not qualify Xbox controller/gameplay behavior.
 */
.text
.globl _start
_start:
    cld
    /* Public LPC47M157 register facts: select logical UART1, base3f8, IRQ4,
     * activate. The serial-off kernel intentionally leaves this disabled. */
    movw $0x2e,%dx
    movb $0x55,%al
    outb %al,%dx
    movl $superio,%esi
    movl $5,%ecx
sio_pair:
    lodsb
    movw $0x2e,%dx
    outb %al,%dx
    lodsb
    incw %dx
    outb %al,%dx
    loop sio_pair
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
    call read_disk
    call report
poll:
    movw $0x3fd,%dx
    inb %dx,%al
    testb $1,%al
    jz idle
    movw $0x3f8,%dx
    inb %dx,%al
    cmpb $'a',%al
    je action
    cmpb $'s',%al
    je save
    cmpb $'?',%al
    je query
    cmpb $'f',%al
    jne poll
    movl $done,%esi
    call text
    jmp query
idle:
    sti
    hlt
    jmp poll
action:
    incl state
    addl $0x13579bdf,cookie
    jmp query
save:
    movl state,%eax
    movl %eax,saved
    movl $0x58424331,sector
    movl %eax,sector+4
    movb $0x30,%bl
    call ata
    movl $sector,%esi
    movw $0x1f0,%dx
    movl $256,%ecx
    rep outsw
    movw $0x1f7,%dx
    movb $0xe7,%al
    outb %al,%dx
    call wait_ata
query:
    call report
    jmp poll
report:
    movl $prefix,%esi
    call text
    movl state,%eax
    call hex
    movl $save_text,%esi
    call text
    movl saved,%eax
    call hex
    movl $cookie_text,%esi
    call text
    movl cookie,%eax
    call hex
    movl $newline,%esi
    call text
    ret
text:
    lodsb
    testb %al,%al
    jz text_end
    call character
    jmp text
text_end: ret
character:
    pushl %eax
    pushl %ecx
    movl $1000000,%ecx
    movw $0x3fd,%dx
tx_wait:
    inb %dx,%al
    testb $0x20,%al
    jnz tx
    loop tx_wait
tx:
    popl %ecx
    popl %eax
    movw $0x3f8,%dx
    outb %al,%dx
    ret
hex:
    pushl %ebx
    pushl %ecx
    movl %eax,%ebx
    movl $8,%ecx
digit:
    roll $4,%ebx
    movl %ebx,%eax
    andb $15,%al
    addb $'0',%al
    cmpb $'9',%al
    jbe emit
    addb $7,%al
emit:
    call character
    loop digit
    popl %ecx
    popl %ebx
    ret
wait_ata:
    movw $0x1f7,%dx
    movl $1000000,%ecx
ata_busy:
    inb %dx,%al
    testb $0x80,%al
    jz ata_ready
    loop ata_busy
ata_ready: ret
ata:
    call wait_ata
    movw $0x1f6,%dx
    movb $0xe0,%al
    outb %al,%dx
    movw $0x1f2,%dx
    movb $1,%al
    outb %al,%dx
    incw %dx
    movb $0xe8,%al
    outb %al,%dx
    incw %dx
    movb $3,%al
    outb %al,%dx
    incw %dx
    xorb %al,%al
    outb %al,%dx
    movw $0x1f7,%dx
    movb %bl,%al
    outb %al,%dx
    call wait_ata
    movl $1000000,%ecx
ata_data:
    inb %dx,%al
    testb $8,%al
    jnz ata_transfer
    loop ata_data
ata_transfer: ret
read_disk:
    movb $0x20,%bl
    call ata
    movl $sector,%edi
    movw $0x1f0,%dx
    movl $256,%ecx
    rep insw
    cmpl $0x58424331,sector
    jne no_save
    movl sector+4,%eax
    movl %eax,saved
no_save: ret
.balign 4
superio: .byte 7,4,0x60,3,0x61,0xf8,0x70,4,0x30,1
.balign 4
state: .long 0
saved: .long 0
cookie: .long 0x31415926
prefix: .ascii "CART "
#ifndef CARTRIDGE_TAG
#define CARTRIDGE_TAG 65
#endif
.byte CARTRIDGE_TAG
.asciz " state="
save_text: .asciz " save="
cookie_text: .asciz " cookie="
newline: .asciz "\n"
done: .asciz "CART COMPLETE\n"
.balign 4
sector: .space 512,0
