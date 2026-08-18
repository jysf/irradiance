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
  id: SPIKE-001
  type: spike                      # epic | story | task | bug | chore | patch | spike
  cycle: spike                     # spike | land  (collapsed from a spec's 5)
  blocked: false
  priority: medium

spike:
  question: Can we decode a Leica Q2 Monochrom DNG bit-exact, and does the oracle discriminate?
                                   # REQUIRED. A spike with no question is just coding.
                                   # For mode: build this is legitimately loose
                                   # ("is a local standup tool worth having?").
                                   # Loose is fine. Absent is not.
  mode: question                   # question | build
  timebox: 3 sessions
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
  id: PROJ-001               # OPTIONAL (null) — a spike may PRECEDE any project
                                   # (that's the point). Back-link at land if one exists.
  # No `stage:` — a spike attaches to the repo, not a stage.
repo:
  id: irradiance

agents:
  explorer: claude-sonnet-4-6  # who ran the spike (tier_map.build; DEC-005)
  created_at: 2026-08-15

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

# SPIKE-001: Can we decode a Leica Q2 Monochrom DNG bit-exact, and does the oracle discriminate?

## Question

**Can `irradiance` decode a Leica Q2 Monochrom DNG bit-exact against an
independent implementation — and does the oracle actually discriminate?**

The second half is the real question. A green oracle that cannot fail is worse
than no oracle: it manufactures confidence. Both halves must be answered before
PROJ-001's specs are written, because every stage after this one is designed
test-first against what this spike establishes.

## Timebox

3 sessions. Hitting it without an answer is **inconclusive** — a real, reportable
result. Do not extend twice.

## Pre-registered questions

Registered BEFORE any code, so the spike cannot quietly redefine success.

**Already answered before the repo existed** — re-verify, do not re-derive
(`docs/oracle-contract.md`, `docs/measured-q2m-dng.md`):

- ~~Are the Leica DNGs compressed?~~ **Uncompressed.** SOF-3 leaves the critical path.
- ~~Is there a CFA?~~ **No** — LinearRaw, `SamplesPerPixel: 1`.
- ~~What does `--raw-checksum` hash?~~ **Uncropped u16 plane, native LE, zero-extended.**
- ~~Does the D750 offer uncompressed NEF?~~ **No** — menu checked.

**Open:**

1. Does *our* parser reach the full-resolution SubIFD and read its tags — not just exiftool's?
2. Does the 14-bit unpack reproduce `StripByteCounts` exactly, and does the plane match
   `dnglab analyze --raw-checksum` **bit-for-bit**?
3. Does the oracle go **RED** on a deliberately broken decode — an injected off-by-one in the
   bit unpacker, a corrupted tag, a wrong black level? *(The crux.)*
4. How visible is `WarpRectilinear` on the Q2M versus `dnglab analyze --srgb`? Decides whether
   it could ever be deferred out of STAGE-003.
5. What SSIMULACRA2 tolerance against the reference render is honest — stated before measuring?
6. Peak memory for a full-res 47 MP develop. (f32 is ~190 MB per plane before anything else.)
7. Does the same DNG produce **byte-identical** output on macOS and Linux?
8. Does `dnglab convert -c uncompressed --embed-raw false` preserve the Fuji 6x6 mosaic
   (`CFARepeatPatternDim: 6 6`), and does it work on a D750 NEF?
9. Confirm the P1100 shoots `.NRW`, 12-bit, uncompressed.
10. Can `dnglab makedng` build a tier-A fixture with an analytically known answer?
11. Measured LOC per module, replacing the estimates in the stage files.
12. **Does the plane path compile for `wasm32-unknown-unknown`?** And what does `no_std` +
    `alloc` cost in error ergonomics across the public API? *(DEC-002 — proposed, not accepted;
    these measurements accept or reject it.)*
13. **What is single-image develop latency WITHOUT in-library parallelism?** If it is bad enough
    that a caller would rather have inner rayon than the browser target, DEC-002 is wrong and
    should be reopened before STAGE-002, not after.
14. **Can `dnglab makedng` build a fixture whose correct output is known by ARITHMETIC** — known
    levels, a known linearization curve — so at least one oracle does not descend from rawler?
    *(See the single-source caveat in STAGE-003.)*
15. **Is a second native-DNG source available cheaply** (a Pixel phone, a borrowed Ricoh GR)?
    PROJ-001 currently validates against one camera, one firmware, one frame — a second source
    is cheap insurance against baking in Leica-specific assumptions.

## What's out of bounds

- **Spike code never becomes product code. This branch is never merged.**
  `test-before-implementation` is a blocking constraint, and retro-fitting tests to
  existing code produces tests that cannot fail.
- No demosaic, no colour, no Bayer — PROJ-002.
- No vendor containers.
- No API design. The spike answers questions; the specs design the library.

## What lands

The durable artifacts are **the corpus and the oracle, not the code**:

- A standing two-tier corpus with its manifest (`docs/conformance-matrix.md`)
- A working three-layer oracle harness, **demonstrated red** on injected faults
- At least one oracle that does NOT descend from rawler (question 14)
- DEC-002 either accepted or reopened, on measurements rather than argument
- Answers to the open questions above, measured and cited
- DECs for anything load-bearing
- Measured LOC replacing the stage-file estimates

## Log

*(no conventions here — it's yours)*

### 2026-08-16 — pre-spike measurements. THE TIMEBOX HAS NOT STARTED.

Everything below was measured while answering "should we start this spike yet?"
It is recorded here so the answers are not re-derived, but **none of the 3-session
timebox has been consumed** — the budget is intact. No spike branch exists and no
decoder code was written.

**Corpus — found, contrary to an earlier conclusion in the same session.**
`~/Pictures/L1021223.DNG`, 86 MB, `LEICA Q2 MONO`. An earlier search reported
"zero RAW files on this machine"; that was **wrong** — one `find` had a syntax
error silenced by `2>/dev/null`, and a second timed out inside the Photos library
before reaching `~/Pictures`. Cited as a live example of DEC-004 rule 1: a
self-report, including this session's own, is a claim to verify rather than trust.
⚠ `~/Pictures/Photos Library.photoslibrary` is **TCC-protected** (`Operation not
permitted` to `ls`/`find`), so "no further files found" is never a safe conclusion
about that path — more frames may exist and must be exported by the maintainer.

**Q10 — `dnglab makedng` tier-A fixture: ANSWERED, with a caveat that changes
STAGE-001.** Yes, a fixture can be built with zero camera files — but **PPM input
only** (TIFF/PGM/PNG/JPEG all rejected), and the output's full-res SubIFD is
`SamplesPerPixel: 3`, `BitsPerSample: 16 16 16`, `Compression: JPEG`.
`--linearization` changes none of it. **There is no makedng path to a 1-sample
monochrome plane**, and its JPEG compression would require lossless JPEG SOF-3 —
out of scope for this project. Tier A can therefore serve the metadata oracle but
not STAGE-002's unpack; hand-built headers become load-bearing. Full write-up in
`docs/oracle-contract.md`.

**Plane contract — re-verified on a second frame.** Not one of the open questions
(it was already marked answered-before-the-repo-existed), but it was cheap and it
is the project's central claim, so it was re-run rather than trusted:

- PGM header `P5 8424 5632 65535\n` = **19 bytes**; stream = **94,887,955** bytes
  = 19 + 8424x5632x2, exactly.
- `--raw-checksum` = `cb653b5bec24d166eef2fd258ee61ac4`, and
  `--raw-pixel | tail -c +20 | dd conv=swab | md5` = **the same string**.

So the contract now rests on two frames rather than one. Still one camera, one
firmware.

**`docs/measured-q2m-dng.md` — one correction.** Every structural value reproduced
on the second frame **except `Orientation`**, which was `Rotate 90 CW` on the first
file and `Horizontal (normal)` on this one. It is a **per-frame** property, not a
camera constant, and listing it as one was a category error. Also newly observed:
`SubIFD2` is a *full-resolution* JPEG preview (8368 x 5584), so raw-IFD selection
must key on `SubfileType` + `Linear Raw`, never on largest dimensions — the margin
is 8424 vs 8368.

**Tooling confirmed present** for the questions that need it: `ssimulacra2` (Q5),
`docker` (Q7 cross-platform byte-identity), `magick` (PPM authoring), `dnglab`
0.7.2, `exiftool` 13.55.

**Still blocked on files not held:** Q8 (needs a Fuji RAF and a D750 NEF) and Q9
(needs a P1100 NRW). Q1/Q2/Q3/Q4/Q6/Q11 need decoder code and are the spike's
actual work.

## Land

*Fill at land. Set `spike.outcome`. Emit DECs for load-bearing choices.*

```
just advance-cycle SPIKE-001 land
just archive-spike SPIKE-001
```
