# SPEC-035 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-035-<cycle>.md`.

## Instructions

- [x] **design** (orchestrator claude-opus-4-8, 2026-11-11) — chart-independent design done; DEC-054
  pins the oracle contract (X-Rite Before/After-Nov-2014 Lab D50 reference, CIEDE2000 in Lab D50,
  one pre-registered exposure scalar over B4/C4/D4/E4, bound mean <= 6.0 / max <= 15.0, red-proof
  mean > 12, committed patch sidecar). Reference + Sharma test files downloaded and SHA-256 pinned.
- [ ] **capture addendum** (orchestrator, design cycle) — when the maintainer's D750 shot lands as an
  uncompressed Adobe-converted DNG: pin it in the manifest, choose the reference set from the chart's
  back-printed date, author + overlay-check the patch sidecar; then unblock and mint the build handoff.
- [ ] **build** — cold claude-opus-5-5 session (handoff minted at the addendum).
- [ ] **verify** — cold, separate session.
- [ ] **ship** — orchestrator; then STAGE-008 stage-ship and the PROJ-002 close ritual.
