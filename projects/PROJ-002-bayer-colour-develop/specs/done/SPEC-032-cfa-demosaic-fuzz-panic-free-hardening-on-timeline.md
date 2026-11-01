# SPEC-032 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-032-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-11-01 on `claude/spec-032-fuzz` (main session).
      The last STAGE-007 spec: a `demosaic` libFuzzer target (mirrors
      `fuzz/fuzz_targets/develop.rs`) + seeds, GRBG/GBRG bit-exact oracle fixtures
      (closes SPEC-030/FU-1 — all four Bayer phases now covered), DC-preservation on
      all four phases, and a tier-A panic-free-on-hostile-CFA test. No `src` change
      (HA is already phase-driven + panic-free). Failing tests enumerated; `cycle: design`.
- [x] **build** — completed 2026-10-31 on `claude/spec-032-fuzz-689903` (cold desktop
      session, `claude-opus-4-8`, HANDOFF-078). Added `fuzz/fuzz_targets/demosaic.rs`
      (+ `[[bin]]`), `demosaic_seeds()` in `examples/fuzz-seeds.rs` (5 seeds), and the
      GRBG+GBRG bit-exact fixtures + DC-on-all-four-phases + `demosaic_is_panic_free_on_hostile_cfa`
      in `tests/demosaic_oracle.rs`. No `src` change. Gates green; nightly fuzz
      11,273,543 runs / 61s / 0 crashes. tokens_total null (desktop `/cost` unavailable —
      recover from transcript at ship). PR to `main`.
- [x] **verify** — ✅ APPROVED — ship at `a7eae9e`. Cold independent review on
      `claude/spec-032-verify-cfbf9c` (`claude-opus-4-8`, HANDOFF-079), retrospective
      (build PR #71 already merged to main). Confirmed TEST+FUZZ only (no `src`);
      all gates green (`just test` demosaic_oracle 23/23, `lint-ci`, `lint-fuzz`
      DEC-027, `fmt`, `validate`, `decisions-audit --changed` clean). Hand-recomputed
      GRBG (3,2)=(135,140,138) + GBRG (1,2)=(94,92,96) from the HA definition — both
      match the pinned asserts + EXPECTED tables; both genuinely drive the
      main-diagonal green phase (closes SPEC-030/FU-1, all four phases now covered).
      Re-ran nightly fuzz: 8,642,384 runs / 46s / 0 crashes. Panic-free test
      non-vacuous (Ok control). tokens_total null (desktop `/cost` unavailable —
      recover from transcript at ship, UUID 0b206331). Ship prompt written;
      `advance-cycle → ship` (verdict approved) run.
- [x] **ship** — completed 2026-11-01 (main session). Build+verify merged (#71 build,
      #72 verify-docs). Reflection + recovered cost totals (build 12,690,825/$28.06 +
      verify 6,804,472/$15.84 = 19,495,297/$43.90, both recovered from the cold
      sessions' transcripts since handbacks were null), `complexity_actual: S`, archive,
      brag. TEST-ONLY → no CHANGELOG (confirmed). `verify_verdict: approved`. **STAGE-007
      now fully shipped (3/3) → run the STAGE-007 stage-ship (Prompt 1d) next.**
