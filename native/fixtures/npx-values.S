/* SPDX-License-Identifier: MIT
 * Original test-owned values/status/constants and non-pop initial-value witness.
 * Integer UART reporting; initial-value witness is stored by the fixture.
 */
.text
npx_values:
    pushfl
    pushal
    cld
    movl $npx_values_prefix,%esi
    call npx_text
    movl $npx_value_addresses,%edi
    movl $17,%ebp
1:  movl (%edi),%eax
    movl (%eax),%ebx
    addl $4,%edi
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
    movb $' ',%al
    call npx_character
    decl %ebp
    jnz 1b
    movb $10,%al
    call npx_character
    popal
    popfl
    ret
.data
.balign 4
npx_restore_status: .long 0
npx_before_save: .quad 0
npx_unit_value: .quad 0
npx_value_addresses:
    .long npx_restore_status, _ParentObserved, _ParentObserved+4
    .long _ObservedPattern, _ObservedPattern+4, _ObservedPattern+8, _ObservedPattern+12
    .long _ParentPattern, _ParentPattern+4, _ParentPattern+8, _ParentPattern+12
    .long npx_before_save, npx_before_save+4, _ParentValue, _ParentValue+4
    .long npx_unit_value, npx_unit_value+4
npx_values_prefix: .asciz "NPX OWN VALUES="
