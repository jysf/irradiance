# SPEC-019 — build dispatch

`HANDOFF-052` is the contract; read it and the spec (`SPEC-019`) in full before
touching code. The design is grounded in a real-file probe (the Q2M carries no
profile tone curve — see the spec's Context), so the curve is settled: the sRGB
OETF (IEC 61966-2-1). Build transcribes it; it does not rediscover it.

Four things that are easy to get wrong here:

1. **Do not touch `develop_into` or `DEC-018`.** The three encoders are an
   additive layer over `develop_into`'s scene-linear `u16` output. No shipped
   function's decoded pixels change — keep it that way, or the develop/levels/
   warp oracles move under you.
2. **Determinism is the point of the table.** `powf` runs exactly once, in
   `examples/gen-tone-table.rs`, producing committed bytes
   (`src/tone_table.rs`). The library calls no transcendental on the encode
   path (AC4). Byte-identity across macOS/Linux follows from the committed
   integer table, not from matching a runtime float (AC7).
3. **The analytic oracle is independent of rawler and must go red.** AC5 derives
   expected values from the sRGB formula in `f64` (±1 in the *test*, not the
   table); AC6 proves a mutated **copy** of the table reds while the unmutated
   table is the negative control — mirror `DEC-017`/`DEC-021`, ship no fault
   switch.
4. **Ship the reader with the field.** Every output mode is read by
   `irr develop --format …` and a test (AC9); the `gen-tone-table` recipe gains
   its AGENTS.md §6 line in the same change. Emit `DEC-025`, add the provenance
   row, and run `just lint-ci` (pinned toolchain) before every push — confirm CI
   is observed green on the shipping SHA, not just locally (AC10).
