# XBX-4: Optional paused-resident Android game sessions

## Status

Future architectural option only. **Not dispatched.** No process pool or setting
is required for XBX-1/XBX-2. Initial support is many durable saved sessions with
one live machine; correct restart/restore must work with zero resident workers.

## Hypothesis

A small number of separate Android runtime processes might keep one additional
game paused in memory and reduce switch latency. This is distinct from letting
several games execute concurrently, from Android Recents tasks, and from pinned
shortcuts. The operating system may reclaim cached processes at any time.

## Before implementing

Measure ordinary checkpoint reload time, active/paused RAM and GPU/JIT allocation,
foreground speed/audio, thermal pressure and durable checkpoint cost. Only build
another resident slot if it has a useful measured benefit within device budgets.

Use statically declared private process slots with dynamic GameId assignment,
one engine and coherent storage owner per slot, validated IPC, safe eviction and
no permanent PID/process name in saved IDs. The two games must never share a
writable virtual-console disk or EEPROM by accident. A profile-wide writer lease
is mandatory where storage is deliberately shared.

Inactive slots pause and release output/input; no false media foreground service,
wake lock or battery-optimization exemption to preserve memory. Eviction or crash
falls back to the durable checkpoint. No promise of infinite resident games.

Test same/different game shortcuts, rapid switches, process death, out-of-space
checkpoint failure, stale state, folds, update/reboot recovery and PSX isolation.
Any user option should describe a best-effort benefit (for example, Keep another
game ready) rather than guaranteed background execution.

iOS may retain the same durable session model without the Android process model.
No unrestricted per-game iOS process or App Store capability is assumed.
