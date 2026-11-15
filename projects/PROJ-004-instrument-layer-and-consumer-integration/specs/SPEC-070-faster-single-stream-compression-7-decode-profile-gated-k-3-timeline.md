# SPEC-070 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-070-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-14 (DEC-065: ScanReader buffers whole destuffed bytes in a u64 and its fill stops silently at a marker or EOF, so errors arise where they do today; HuffTable gains 11-bit lookup and fast (code plus difference) tables; today's bit-at-a-time path stays for long codes, the scan end and errors; restart gives back unread whole bytes; profile: 7676/8395 samples in the entropy decode; probe: K-3 III 708 -> 108 ms, Typ 246 606 -> 95, Pixels about 320 -> 60, byte-identical on all 5 files, and identical Results on 28,880 corrupted real streams and 21,000 synthetic restart streams; targets K-3 III <= 150 ms and Typ 246 <= 140 ms; HANDOFF-147 build ready)
- [ ] **build** — HANDOFF-147
- [ ] **verify**
- [ ] **ship**
