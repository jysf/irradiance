# SPEC-071 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-071-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-16 (bundles the four STAGE-020 [XS] follow-ups; probe measured exactly 10 allocations per Compression 7 strip or tile on both tier-A fixtures (40 for 4 strips, 120 for 12 tiles); no src change; build before SPEC-072, which also edits AGENTS.md section 6; HANDOFF-149 build ready)
- [x] **build** — completed 2026-11-16 (HANDOFF-149, PR #247; absolute pin of 10 allocations per C7 strip or tile, red-proved on an archived copy; fuzz-ljpeg recipe; AGENTS.md section 6 now lists all 35 recipes; just test 697/0/1; all gates green; floor 5,077,732 tok / $4.05)
- [x] **verify** — completed 2026-11-16 (HANDOFF-151; APPROVED at 9bb96de, no code changes; explanation of the 10 confirmed in parse_headers; AC1 red-proof reproduced and 9 own mutants as predicted; fuzz-ljpeg from a missing corpus dir 2.03M runs clean; 35 of 35 recipes in section 6; just test 697/0/1; all gates green; CI 44/44; floor 3,350,892 tok / $2.72)
- [x] **ship** — completed 2026-11-16 (PR 247 merged as d49afd9; true cost re-summed: build 6,883,272 / $5.20, verify 4,323,610 / $3.45, total 11,206,882 / $8.65 against a 12M estimate; complexity_actual S)
