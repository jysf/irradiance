# SPEC-027 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-027-<cycle>.md`.

## Cycles

- [x] **design** (2026-10-30) — approach + failing tests. Register both Bayer DNGs
  in `tests/corpus/manifest.toml`, fold SPEC-025's direct-path Bayer oracle onto the
  manifest (add to `DECODABLE`, delete the bespoke constants/test, manifest-source the
  red-proof), migrate the SPEC-028/029 plane_unpack corpus tests to `manifest.get()`,
  and add a synthetic CFA fuzz seed + a tier-A panic-free-on-CFA test. Three counts
  interlock (`MANIFEST_ENTRIES` / `DECODABLE` / skip-partition = 9 / 6 / 3). Expected M.
- [ ] **build** — make the Failing Tests pass (cold session).
- [ ] **verify** — independent cold review.
- [ ] **ship** — archive, Reflection, cost totals, brag.
