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
  65.4M tokens. PR #52.
- [x] **verify** (2026-10-30) — ✅ APPROVED at e95ec04 (PR #52, retrospective cold
  review). Confirmed TEST/CORPUS/FUZZ-only (no src/); recomputed both sha256 +
  both dnglab raw-checksums == manifest (9/9 verified live); oracle fold is
  manifest-driven (DECODABLE 6, partition 6+3==9, no hard-coded Bayer checksum
  const, red-proof reddens from the manifest reference honest b8e8025d != mutant);
  plane_unpack migrated; 3 CFA fuzz seeds idempotent; CFA parse panic-free
  (well-formed→Cfa, truncated→Truncated, hostile→costed per DEC-012/DEC-015). All
  three build deviations adjudicated HONEST. Gates: just test 0 failed, lint-ci
  (clippy 0.1.98) / fmt / validate / ready / decisions-audit all exit 0; bounded
  ifd+plane fuzz over the CFA seed 0 crashes. HANDOFF-073; 9.9M tokens. PR #53.
- [x] **ship** (2026-10-30) — Reflection, cost totals (build + verify = 75,323,789
  tokens / $169.20; ship un-metered), timeline, archive, brag. `verify_verdict:
  approved`, `complexity_actual: M` (as expected). Test/corpus/fuzz only — no
  CHANGELOG. Worst-defect `build` (a design under-specification, handled correctly).
