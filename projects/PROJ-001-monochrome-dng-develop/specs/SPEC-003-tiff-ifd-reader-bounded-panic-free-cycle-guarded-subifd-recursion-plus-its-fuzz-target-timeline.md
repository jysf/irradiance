# SPEC-003 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-003-<cycle>.md`.

## Instructions

- `[x]` **design** — 2026-08-20, `ee5f310`. Designed the reader and, more
  usefully, **found and fixed the `cargo fuzz` blocker before build started**:
  the inner bare `"cargo" "build"` resolves to Homebrew's stable cargo and
  rejects `-Zsanitizer`, so the shim must be first on `PATH`. Proven end to end
  (init works, 32.9 M execs in 16 s, a planted unchecked index caught with exit
  77), which is why acceptance criteria 4 and 5 were known-achievable rather
  than hoped for.
- `[x]` **build** — 2026-08-20, `b79c7ef` on `feat/spec-003-ifd-reader`, not
  merged. HANDOFF-011. All 7 acceptance criteria met; nine gates green; both
  fuzz directions pasted in the handback. No `#[allow]` of any policy lint was
  needed. New: `DEC-011`. Two measured corrections to the spec's own notes —
  only **one** corpus file is big-endian, and `K3III.PEF` has no `SubIFD` at
  all (its plane is in IFD0, with no `NewSubfileType` tag). 10,967,269 tokens,
  deduped by `message.id`.
- `[x]` **verify** — 2026-08-20, HANDOFF-012, reviewing `b79c7ef` at `644815f`.
  ⚠ **PUNCH LIST** — one ship-blocker, documentation and config only, no `src/`
  change. Nine gates re-run and green; both fuzz directions run by the reviewer,
  with the planted fault at a **different site** from the build's and its
  lint-cleanliness measured as a negative control first (`just lint` **and**
  `just lint-no-allow` both exit 0 with the fault in) — libFuzzer then found it
  from a **zero-seed** corpus in ~38,900 execs. Ship-blocker: `DEC-011`'s licence
  table records `libfuzzer-sys` as `MIT OR Apache-2.0` where it declares
  `(MIT OR Apache-2.0) AND NCSA`, omits three crates from the graph (one carrying
  an LGPL option), and rests on the premise that `cargo deny` cannot reach
  `fuzz/` — which `--manifest-path fuzz/Cargo.toml` disproves. Substance is fine;
  the record is not. Eight follow-ups, including a **third** wrong corpus fact in
  the spec ("three JPEG-compressed" — two are JPEG, the PEF is 65535) and
  `docs/conformance-matrix.md` missing rows for three held bodies.
  9,036,505 tokens, deduped by `message.id` (1.61x).
