# SPEC-033 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-033-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-02 (PR #77; minted DEC-033)
- [x] **build** — HANDOFF-080 (delegated build), branch `claude/spec-033-build-f223ec`
       PR #79, cost null (desktop `/cost` unavailable — recover from transcript at ship), completed 2026-11-02
- [x] **verify** — HANDOFF-081 (cold verify), ✅ APPROVED at `bb4aab11`, completed 2026-11-02.
       Independently re-derived the bit-exact oracle (own Python reference, standards-transcribed
       `S`/`B`, exact f64 order → matches the real kernel to the bit); proved the oracle bites via
       2 real-kernel mutations (perturbed `S[0][0]`; removed the WB `1/asn`); `irr color` end-to-end
       on Canon (FM1) + Nikon inv(CM1) → valid P6 PPM, genuine colour; all gates green.
- [ ] **ship** — prompt: `prompts/SPEC-033-ship.md`
