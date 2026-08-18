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
  cycle: land                      # spike | land  (collapsed from a spec's 5)
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
  outcome: answered                # set at LAND. Never leave null on a landed spike —
                                   #   answered     — the question got an answer
                                   #   inconclusive — timebox hit, no answer (a real result)
                                   #   graduated    — the code becomes real work (see below)
                                   #   discarded    — the code is thrown away (also a win)
  landed_at: 2026-08-18            # YYYY-MM-DD, stamped at land

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
  decisions: [DEC-004, DEC-005]    # DECs EMITTED AT LAND (not during — that's the point)

# Cost is ADVISORY for spikes — `just cost-audit` does NOT gate them (DEC-012 v1).
# A spike is often pre-project and deliberately cheap; a cost gate on exploration
# is exactly the friction that would make people skip the artifact.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0.00
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

### 2026-08-18 — session 1 of 3. Q1, Q2, Q3, Q6, Q11 answered.

Throwaway decoder on branch `spike/001-q2m-bit-exact`, in `spike-001/`.
**Zero dependencies, pure std** — which answers a question nobody asked: the
container reader and unpacker need no crates at all. MD5 comparison is done by
the shell, not linked in.

**Q1 — does OUR parser reach the full-resolution SubIFD? YES.**
Walks IFD0's chain and recurses `SubIFDs` (tag 330), depth- and cycle-guarded.
Finds 4 IFDs on a Q2M file and selects the sensor plane on
`NewSubfileType == 0 && Photometric == 34892 (LinearRaw) && SamplesPerPixel == 1`
— deliberately **not** on largest dimensions, because SubIFD2 is a full-res JPEG
preview only 56 px narrower. Every tag matched exiftool: 8424x5632, 14-bit,
uncompressed, strip offset 2769920, levels 512/16383, ActiveArea, DefaultCrop,
both opcode lists.

**Q2 — bit-exact? YES, ON THE FIRST ATTEMPT, ON ALL THREE FRAMES.**

| Frame | our plane md5 | `dnglab --raw-checksum` |
|---|---|---|
| L1021223 | `cb653b5bec24d166eef2fd258ee61ac4` | identical |
| L1026016 | `3f1851259f3119c0a2fa98d84065f2af` | identical |
| L1026192 | `c7348179f042d9597be7829d03fa5d8a` | identical |

47,443,968 samples each, zero differing bytes. The layer-0 arithmetic closed
first (`8424 x 5632 x 14 / 8 == StripByteCounts`), and the documented
representation — MSB-first packed, native-LE u16 out, zero-extended, uncropped,
no black subtraction — was correct exactly as written. **The project's central
technical risk is retired.**

**Q3 — does the oracle go RED? THE CRUX, AND THE ANSWER IS QUALIFIED.**

| Injected fault | Oracle |
|---|---|
| off-by-one in the bit cursor | ✅ **RED** — `4ce94344…` vs `cb653b5b…` |
| corrupted `ImageWidth` tag | ✅ **RED at layer 0**, before any pixel is decoded |
| **wrong `BlackLevel` (+64)** | ⚠️ **STAYED GREEN** |

**The plane oracle is structurally blind to a levels fault, and this is the most
valuable thing the spike has produced.** It is not a bug — it follows from the
verified contract: `--raw-checksum` hashes the plane with *no black subtraction*,
so by construction it cannot see a wrong `BlackLevel` or `WhiteLevel`.

The consequence is concrete and lands on STAGE-002. Its backlog item *"Black/white
level normalization, ActiveArea → DefaultCrop, and orientation"* has **no oracle
coverage from the plane layer**. Ship it believing the checksum covers it and a
wrong black level sails through green — which is precisely the
`oracle-must-be-shown-red` failure mode, found in a layer nobody had checked.
**Levels, crop and orientation need their own oracle** (the `--srgb` develop layer,
or a synthesized fixture with analytically known levels). This should be a DEC at
land, and it changes STAGE-002's spec breakdown.

**Q6 — peak memory: 264 MB** (277,331,968 bytes RSS) for a full 47 MP unpack, in
0.10 s wall. That is file (85.8 MB) + plane (94.9 MB) + the emit copy (94.9 MB);
a real decoder that streams its output drops to ~181 MB. Well inside a sane budget
for the *unpack*, but the brief's tiling risk stands for the develop path — an f32
plane is ~190 MB on its own.

**Q11 — measured LOC**, code lines only (blank and comment excluded):

| Module | code |
|---|---|
| `tiff.rs` — IFD walk, SubIFD recursion, guards | 117 |
| `dng.rs` — tag model + sensor-plane selection | 72 |
| `unpack.rs` — layer-0 check + 14-bit unpack | 40 |
| `main.rs` — driver (not product code) | 84 |
| **container + decode, excluding driver** | **229** |

Against the stage files' 550–700 for the mono path on top of the container reader.
**Read this as encouraging but not yet a refutation** — the spike implemented the
tractable part and omits fuzz hardening, a real error taxonomy, both opcodes,
levels/geometry, tone curve, output modes, and every test. The honest claim is
narrower: *the container reader and 14-bit unpack together are ~230 lines, not
~500*, and they need no dependencies.

**Still open:** Q4 (WarpRectilinear visibility), Q5 (SSIMULACRA2 tolerance —
must be stated before measuring), Q7 (macOS/Linux byte-identity; docker is
installed). **Blocked on files not held:** Q8, Q9.

Timebox: 1 of 3 sessions used.

### 2026-08-18 — session 1 continued. Q4, Q5 answered. Q7 blocked.

**Q4 — how visible is WarpRectilinear? IT CANNOT BE DEFERRED. ~504 px at the corner.**

`OpcodeList3` parsed straight out of the file (big-endian opcode stream,
`WarpRectilinear` = opcode id 1, version 1.4.0.0, 68 param bytes):

```
planes 1                      <- monochrome: one plane, as expected
kr0  0.999251106              kt0 0.0        <- no tangential term at all
kr1 -0.06137651287726358      kt1 0.0
kr2 -0.09391554139336016
kr3  0.05588200921529175      optical centre cx 0.5, cy 0.5 (dead centre)
```

`f(r) = kr0 + kr1r² + kr2r⁴ + kr3r⁶`, radial displacement over the 8368x5584
crop (half-diagonal 5030 px):

| r | f(r) | displacement |
|---|---|---|
| 0.00 | 0.999251 | −0.0 px |
| 0.50 | 0.978910 | −53.0 px |
| 0.75 | 0.944957 | −207.7 px |
| 1.00 | 0.899841 | **−503.8 px** |

A 10% inward correction at the corner — **6% of image width**. Pure radial,
no tangential. This settles the STAGE-003 question: skipping `WarpRectilinear`
does not produce a slightly-off image, it produces a visibly wrong one, and no
reference render would ever match. Consistent with the Q-series 28 mm lens being
designed around software correction.

*(Assumes the DNG convention that r normalizes to 1.0 at the corner. The pixel
figures should be confirmed against the spec before being quoted as exact; the
magnitude class is robust either way, and the empirical check below agrees.)*

**`OpcodeList1`** parsed too: `FixBadPixelsConstant`, constant 0, bayerPhase 2.

**⚠ Q5 prerequisite — `dnglab analyze --srgb` IS MALFORMED FOR MONO FILES.**
The help says "16-bit sRGB TIFF". It is not a TIFF. It is a PNM — and on a
monochrome file it writes a **`P6` (RGB) header over a `P5` (grayscale)
payload**: exactly `w*h*2` bytes where the header declares `w*h*3*2`. One third
of the promised data. Reproduced twice, byte-identical size both times
(93,453,843 = 19-byte header + 8368*5584*2).

Any standard PNM reader either errors or reads garbage, so **oracle layer 3 as
written in `docs/oracle-contract.md` does not work on the files this project
targets.** The workaround is one line — rewrite the header as
`P5 8368 5584 65535` and the payload is a valid 16-bit grayscale image (verified:
decodes to a real photograph, mean 0.237, stddev 0.114). This must be in the
STAGE-003 spec, not discovered again.

**Q5 — SSIMULACRA2 tolerance. PRE-REGISTERED ≥ 85 BEFORE MEASURING, and it holds
— with a scope limit that matters more than the number.**

Pre-registered, in writing, before any score was computed: **≥ 85** (the tool's
own "impossible to distinguish in a flip test at 1:1"), on the reasoning that
100 is unrealistic when our tone curve and dnglab's differ by implementation.
Falsifier registered at the same time: **a missing warp must land far below 85**,
or the metric cannot catch what we most need it to.

Calibration, perturbing dnglab's own render at quarter resolution
(2092x1396 — absolute scores are resolution-dependent; treat as relative):

| perturbation | score | verdict at 85 |
|---|---|---|
| identical | **100.00** | sanity check passes |
| gamma 1.01 | 95.03 | passes |
| gamma 1.05 | 88.51 | passes (just) |
| 1-pixel shift | 62.96 | **caught** |
| missing warp | **−68.05** | **caught, emphatically** |

The falsifier is satisfied: a missing warp scores −68. Even a *one-pixel* shift
scores 63. **Geometry errors cannot hide from this metric at an 85 bar.**
*(Caveat: the missing-warp simulation is a 2-term fit and leaves unfilled edges,
which inflates the penalty. The 1-px shift result carries the conclusion on its
own.)*

**🔴 THE FINDING THAT MATTERS — neither oracle covers levels.**

Session 1 established that the plane oracle is *structurally* blind to a
`BlackLevel` error. The obvious assumption is that the develop oracle covers it.
**It does not.** Simulating a real black-level error as the affine change it
actually is — `y = ax + b` where `a = 15871/(16383−B)`, `b = (512−B)/(16383−B)`:

| BlackLevel used instead of 512 | score | at the 85 bar |
|---|---|---|
| 513 (+1) | 100.00 | passes |
| 528 (+16) | 100.00 | passes |
| 576 (+64) | 95.62 | passes |
| 768 (+256) | 87.51 | **passes — a 50% levels error** |
| 1024 (+512) | 73.16 | caught |

**A black-level error must be ~50% of the entire black level before the develop
oracle notices, and a 100% error still only scores 73.** Both layers are blind
in the same place, for different reasons: the plane oracle by construction (no
black subtraction), the develop oracle because SSIMULACRA2 is a *perceptual*
metric and a levels error is very nearly an affine tone change — which it is
designed to forgive.

**Consequence: levels must be verified by ANALYTIC ASSERTION, not by any image
comparison.** Read `BlackLevel`/`WhiteLevel` from the file and assert the
normalization maps black→0 and white→1 exactly, plus a synthesized fixture with
known levels. No perceptual metric belongs anywhere near this. This is a DEC at
land and it adds a spec to STAGE-002.

The three-layer oracle is sound for what it covers — container, plane geometry,
render geometry — and has a **hole exactly where the two layers meet**. That hole
is what this spike was for.

**Q7 — BLOCKED, not answered.** Docker is installed but the daemon is not
running (`no such file or directory` on the socket). Needs Docker Desktop started;
the check itself is cheap once it is.

Timebox: still 1 of 3 sessions used.

## Land

**Outcome: `answered`.** Both halves of the question are answered, and the second
half — *does the oracle actually discriminate?* — turned out to be the one worth
asking.

**Can we decode a Q2 Monochrom DNG bit-exact? YES.** First attempt, all three
held frames, 47,443,968 samples each, zero differing bytes against
`dnglab analyze --raw-checksum`. 229 code lines of container reader + unpacker,
**zero dependencies, pure std**. The documented plane representation was correct
exactly as written. PROJ-001's central technical risk is retired.

**Does the oracle discriminate? PARTLY — AND THE GAP IS THE DELIVERABLE.**
A bit-cursor off-by-one goes red. A corrupt width tag goes red at layer 0 before
any pixel decodes. A missing warp scores −68 on the develop layer; even a
*one-pixel* shift scores 63. But **a wrong `BlackLevel` passes both layers** — the
plane oracle by construction (it hashes without black subtraction), the develop
oracle because SSIMULACRA2 is perceptual and a levels error is nearly an affine
tone change it is designed to forgive. A **50% levels error scores 87.5** and
sails through an 85 bar.

Had this spike not run, STAGE-002 would have shipped levels normalization with no
oracle coverage, believing it had two. That is the `oracle-must-be-shown-red`
failure mode arriving through the front door, and finding it is worth more than
the decoder was.

### DECs emitted

- **`DEC-004`** — levels, crop and orientation are verified by **analytic
  assertion**, never by image comparison. Adds a spec to STAGE-002.
- **`DEC-005`** — the develop oracle reads `--srgb` as **P5** (dnglab writes a P6
  header over a P5 payload on mono files — its bug, our one-line workaround), and
  its tolerance is **SSIMULACRA2 ≥ 85**, pre-registered before measuring, scoped
  to geometry and gross tone only.

### The code's fate: DISCARDED, deliberately

`spike-001/` stays on the unmerged branch `spike/001-q2m-bit-exact` and **is never
merged**. `test-before-implementation` is a blocking constraint; retro-fitting
tests to a working decoder produces tests that cannot fail — the same defect this
spike exists to detect. STAGE-001 and STAGE-002 re-derive it test-first. The
branch remains as evidence that the approach works and as a reference for the
measurements above.

### What this changes downstream

- **STAGE-002** gains a levels/geometry spec with an analytic oracle (DEC-004),
  and must not assume the plane checksum covers normalization.
- **STAGE-003** must implement `WarpRectilinear` — it is **not deferrable** at
  ~504 px of corner displacement — and its develop oracle must apply DEC-005's
  P5 workaround. `docs/oracle-contract.md` layer 3 was wrong and is corrected.
- **STAGE-001** should note that the container reader plus unpacker measured ~230
  lines without dependencies, against a 550–700 estimate for the whole mono path.
  Encouraging, not yet a refutation — the spike omitted fuzz hardening, a real
  error taxonomy, both opcodes, levels/geometry, tone curve, output modes and all
  tests.

### Unanswered, carried to SPIKE-002

Q7 (cross-platform byte-identity — premature: the decode path has no floating
point, so the question only becomes real once warp resampling and the tone curve
exist), Q8 and Q9 (both blocked on camera files not held). See
`spikes/SPIKE-002-*.md`.

**Timebox: 1 of 3 sessions used.** Landing early rather than spending the
remainder on questions that are blocked or premature.
