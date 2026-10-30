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
- [x] **build** (2026-10-30) — Failing Tests pass; all gates green (just test, fmt,
  lint-ci, validate, decisions-audit) + bounded ifd/plane fuzz 0 crashes. Both Bayer
  DNGs registered (measured bytes/sha256), oracle folded onto the manifest (6/3/9),
  plane_unpack migrated, CFA fuzz seed + tier-A panic-free CFA tests added. HANDOFF-072;
  65.4M tokens. PR #PR_PLACEHOLDER.
- [ ] **verify** — independent cold review.
- [ ] **ship** — archive, Reflection, cost totals, brag.
