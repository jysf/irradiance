---
# A SPIKE is a BOUNDED EXPLORATION — the phase before you know the shape.
# Two modes, one discipline:
#   mode: question — a timeboxed investigation. Code is evidence, usually thrown away.
#   mode: build    — a vibe-coding session. Code is the deliverable, you intend to keep it.
# See AGENTS.md "Spike lane" and docs/decisions/DEC-012.
#
# Collapsed cycle: spike -> land.
#   spike — explore. NO spec, NO failing tests, NO DEC required. Speed IS the value.
#   land  — MANDATORY. The entire point of the lane: write down what you learned,
#           emit the DECs the exploration already made, decide the code's fate.
#
# There is deliberately NO verify step: a spike has no acceptance criteria, so an
# "independent verify" would have nothing to check (DEC-012). The timebox and the
# mandatory land step are the disciplines that replace it.

task:
  id: SPIKE-002
  type: spike                      # epic | story | task | bug | chore | patch | spike
  cycle: spike                     # spike | land  (collapsed from a spec's 5)
  blocked: true                    # PREREQUISITES UNMET — see ## Blocked on
  priority: medium

spike:
  question: Does the monochrome path generalise beyond one Leica body, and what do other containers require
                                   # REQUIRED. A spike with no question is just coding.
                                   # For mode: build this is legitimately loose
                                   # ("is a local standup tool worth having?").
                                   # Loose is fine. Absent is not.
  mode: question                   # question | build
  timebox: 2 sessions
                                   # REQUIRED. Exceeded means STOP and land it as
                                   # `inconclusive` — not extend. Extending twice
                                   # means it isn't a spike, it's an unframed project.
  outcome: null                    # set at LAND. Never leave null on a landed spike —
                                   #   answered     — the question got an answer
                                   #   inconclusive — timebox hit, no answer (a real result)
                                   #   graduated    — the code becomes real work (see below)
                                   #   discarded    — the code is thrown away (also a win)
  landed_at: null                  # YYYY-MM-DD, stamped at land

project:
  id: null               # OPTIONAL (null) — a spike may PRECEDE any project
                                   # (that's the point). Back-link at land if one exists.
  # No `stage:` — a spike attaches to the repo, not a stage.
repo:
  id: irradiance

agents:
  explorer: claude-sonnet-5  # who ran the spike (tier_map.build; DEC-005)
  created_at: 2026-08-18

references:
  decisions: []                    # DECs EMITTED AT LAND (not during — that's the point)

# Cost is ADVISORY for spikes — `just cost-audit` does NOT gate them (DEC-012 v1).
# A spike is often pre-project and deliberately cheap; a cost gate on exploration
# is exactly the friction that would make people skip the artifact.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPIKE-002: Does the monochrome path generalise beyond one Leica body, and what do other containers require?

## Question

SPIKE-001 proved `irradiance` can decode **one camera's** monochrome DNG
bit-exact. Everything it established — the container walk, the tag model, the
14-bit unpack, the oracle contract — rests on **one body (serial 5597430), one
firmware, three frames.** This spike asks whether any of that is Leica-specific.

Three sub-questions, carried forward from SPIKE-001 or newly raised by it:

1. **Does the container reader work on a monochrome DNG from a different make?**
   Candidates identified in the raw.pixls.us mirror: Leica M Monochrom, Leica
   M Monochrom (Typ 246), Pentax K-3 Mark III Monochrome. The Pentax ships
   **both a DNG and a native PEF of the same scene**, which is a free
   cross-container comparison.
2. **Do other bit depths and packings hold?** SPIKE-001 only ever saw 14-bit
   tightly packed with zero row padding, where `width*14/8` happened to land on
   a byte boundary exactly. A 12-bit source (Nikon P1100 NRW) is the cheapest
   test of a hardcoded assumption — SPIKE-001's unpacker takes `bits` as a
   parameter but has **never been run with any value but 14**.
3. **Does `dnglab convert -c uncompressed` preserve what we need?** SPIKE-001's
   Q8, unanswered. Specifically whether it preserves a Fuji X-Trans 6x6 mosaic
   (`CFARepeatPatternDim: 6 6`) and whether it works on a D750 NEF. This decides
   whether `convert` is a viable corpus-widening tool or a trap.

## Timebox

2 sessions. `inconclusive` is a real result — see SPIKE-001, which landed at 1 of
3 and was more valuable for stopping early than it would have been for grinding
on blocked questions.

## ⛔ Blocked on — the prerequisite list

**Do not start this spike until these are done.** Every one of them needs a human
or a network, not an agent, and running the spike without them burns the timebox
on questions that cannot be answered.

- [ ] **Resolve licences for the raw.pixls.us samples.** The `data.lfs` clone at
      `../data.lfs` carries **no licence metadata at all** — no LICENSE file,
      nothing per-file. Licences live in the raw.pixls.us *web* database. Under
      `DEC-003` a file with no recorded licence cannot enter the manifest, so this
      blocks every third-party sample. Look up the three monochrome candidates
      and record `CC0` vs `CC-BY-*` vs `CC-BY-NC-SA` per file.
- [ ] **Pull the files that come back usable.** They are LFS *pointers* today
      (133 bytes each), which is why the clone is only 9 MB:
      `git -C ../data.lfs lfs pull --include="Leica/M Monochrom/*"`.
      Sizes are already known from the pointers — M Monochrom 36.4 MB,
      M Monochrom Typ 246 21.7 MB, Pentax K-3 III Mono 37.7 MB DNG + 37.4 MB PEF.
- [ ] **Add each pulled file to `tests/corpus/manifest.toml`** with its licence,
      source URL, sha256 (the LFS pointer already states it — no download needed
      to record it) and its `dnglab analyze --raw-checksum`.
- [ ] **A 12-bit source.** Nikon P1100 `.NRW` is the target (`conformance-matrix.md`
      calls it the easiest Bayer source owned, and 12-bit against the Leica's
      14-bit catches hardcoded bit-depth assumptions). Either shoot one, or find
      a CC0 12-bit sample. **Not held today.**
- [ ] **A Fuji RAF and a Nikon D750 NEF** for sub-question 3. **Not held today.**
      The maintainer has offered to ask people with these bodies.

## Explicitly NOT in this spike

- **Q7, cross-platform byte-identity — deliberately dropped, not deferred.**
  SPIKE-001 verified the decode path contains **zero floating point**, no
  `HashMap` iteration order (it uses `BTreeMap`), and no platform-dependent width
  beyond `usize`. The single native-endian call (`to_ne_bytes`) is native *by
  design*, to match the oracle's native-LE contract. On any little-endian 64-bit
  platform the output is byte-identical **by construction**, so testing it now
  would confirm what inspection already shows.

  The question becomes real when **floating point arrives** — warp resampling and
  the tone curve, both STAGE-003. It belongs there, as a CI job on
  `ubuntu-latest` (free) or a `wasm32` target run (already installed, and
  *stronger* than Linux-on-arm64 because it changes pointer width to 32-bit).
  Docker is not needed for it and never was. `DEC-002` already proposes the
  mitigations (table-driven tone curves over `powf`, pinned reduction order, no
  runtime SIMD dispatch).
- Demosaic, colour, and anything Bayer — that is PROJ-002's own framing.
- Lossless JPEG SOF-3 — PROJ-003.
- Any decoder code that is meant to survive. As with SPIKE-001, code here is
  evidence and its branch is never merged.

## What lands

- A per-camera answer on whether the container walk and tag model hold unmodified
- The first non-14-bit unpack, or a recorded reason there wasn't one
- A verdict on `dnglab convert` as a corpus-widening tool
- Manifest rows, with licences, for every file obtained
- `docs/conformance-matrix.md` rows updated from measurement rather than
  from a vendor's camera list

## Log

*(no conventions here — it's yours)*

## Land

*Fill at land. Set `spike.outcome`. Emit DECs for load-bearing choices.*

```
just advance-cycle SPIKE-002 land
just archive-spike SPIKE-002
```
