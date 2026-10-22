# XBX-3: Complete-install firmware and mobile-store release gates

## Status

Release/readiness tracker, not authority for store submission or paid accounts.
Relevant source/firmware investigation is already in scope in XBX-1. Do not block
all Android integration on complete iOS readiness or treat this tracker as an
unbounded campaign.

## Goal

A coherent installation that includes every component we can legitimately and
reliably distribute. Games remain user-provided. Extra original firmware is
conditional on genuine compatibility needs, not a design assumption.

## Required decisions and proof

1. Audit and source-build a redistributable boot/kernel route and empty/base disk
   where viable. Roswell/OXOS are candidates, not accepted dependencies. Do not
   package comparison images containing Microsoft components or private EEPROM
   data. Record exact source, fetched inputs, license and generated image identity.
2. Prove gameplay, ordinary saving and complete-state recovery with the selected
   replacement. Bind firmware to sessions. Preserve supplied firmware and never
   silently swap a running/saved session to a different kernel.
3. Keep complete license/copyright and corresponding-source delivery for the
   actual combination. Review any GPLv2-only versus GPLv3 dependency conflict;
   copying PSX's complete donor bundle is not acceptable by assumption.
4. Verify Android native page-size/runtime and store packaging, then-current SDK,
   scoped storage, accessibility, privacy, ratings and update/signing recovery.
   A fork's Play listing is not acceptance of xbx.
5. Evaluate stock-iOS CPU execution and graphics with normal signing on a physical
   device when an authorized toolchain/device exists. No sideload/JIT debugger
   condition is ordinary App Store compatibility. Keep failure boundaries exact.
6. Review original branding and store materials. No copyrighted game media,
   platform logos, proprietary dashboard or implied endorsement.

Public binary distribution/store submission occurs only after a separate explicit
release decision and its actual checks. Do not enroll accounts, pay for services,
change security policy or upload private owner content under this issue.
