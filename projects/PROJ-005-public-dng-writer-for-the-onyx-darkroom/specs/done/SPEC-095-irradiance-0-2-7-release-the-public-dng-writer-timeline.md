# SPEC-095 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-095-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-11-25 (STAGE-032 framed after the STAGE-017 close, PR 342)
- [x] **design** — completed 2026-11-25 (0.2.7 patch, semver clean on 153f517 with and without all features; docs.rs metadata; writer-only, PR 343 held)
- [x] **build** — completed 2026-11-25, PR 345 (HANDOFF-197; 13.6M tokens floor, about 9.52 USD; semver clean both ways, Error 40 bytes before and after, packaged-crate writer check green with dng_validate on all three framings; prep-complete, not tagged or published)
- [x] **verify** — completed 2026-11-25, APPROVED at 252360b (HANDOFF-199; 7.9M tokens floor, about 6.08 USD; own packaged-crate consumer on the M Monochrom Typ 246 with EXIF, XMP, tags and preview, deterministic, dng_validate clean; wording fixes only)
- [x] **ship** — 2026-11-25 (v0.2.7 tagged on a360b4f and published by the user; clean-host checks passed; 21,500,682 tokens / about $15.60 across build and verify, design and ship in the orchestrator session)

Markers: `[ ]` not started · `[~]` in progress · `[x]` complete
· `[?]` blocked (one-line reason, needs human or external unblock).
