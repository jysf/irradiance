# SPEC-073 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-073-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-16 (version 0.2.2, a compatible patch release, chosen by the user after the design's public-API audit of v0.2.1..main: no removed or changed public item, new fields only on non_exhaustive types, no new dependency or feature, MSRV 1.90 unchanged; the build installs cargo-semver-checks (user-approved) and stops on any breaking report; Phase 1 prep only, the user tags and publishes; HANDOFF-153 build ready)
- [x] **build** — HANDOFF-153: completed 2026-11-16, PR #253 (Phase 1 prep, unmerged; stopped once on the cargo-semver-checks discriminant flag on Error, accepted by the user as a false positive; 0.2.2, CHANGELOG, release notes, four doc fixes, 12/12 corpus, 710 passed, dry-run clean, all gates green; 10,790,137 tok floor, $9.04)
- [x] **verify** — HANDOFF-154: completed 2026-11-18, post-release, APPROVED at 46b6fc2 (crates.io 0.2.2 checksum and .cargo_vcs_info sha match the v0.2.2 tag and the #253 merge; 19-file list identical to cargo package --list, sources byte-identical, no self dev-dependency; semver re-run 201/202 with the only failure the Error discriminant lint, and a registry probe crate proves an as-cast is E0605, so it is unobservable; CHANGELOG [Unreleased] promoted verbatim, release notes and Known limitations hold in code; just test 12/12 corpus, 710 passed, 184 s; lint-ci, fmt-check, doc-check, deny, msrv green; publish dry-run says already uploaded, as expected; follow-ups: a stale develop_into tone sentence, an optional GitHub release; floor 6,654,906 tok / $5.00)
- [x] **ship** — completed 2026-11-18 (v0.2.2 published 2026-11-17 by the user, tag v0.2.2 at 46b6fc2 matching the crate's vcs sha; registry smoke test green on K-3 III, Q2M and Pixel 4a; true cost build 12,473,090 / $10.14, post-release verify 7,627,360 / $5.70, total 20,100,450 / $15.84 against a 10M estimate; complexity_actual S)
