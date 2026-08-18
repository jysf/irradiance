---
source: "irradiance — claude-plus-agents instance, scaffolded 2026-08-15 (template v0.6.38)"
captured_at: 2026-08-15
captured_by: claude
status: open                # open | addressed | deferred
---

# Template instantiation friction — a library, and the first live claude-plus-agents instance

Captured at scaffold time, not at project close, because **instantiation friction
is only observable once and then it is gone.** Every finding below was hit while
turning the template into `irradiance`; each is cited to a file and line in the
template as shipped at `VERSION` 0.6.38, and each was verified by running the
thing rather than reading about it.

Two things make this instance worth harvesting beyond the usual:

1. **It is a library, not an app.** Every full-tier instance in
   `docs/harvests/instances.md` is an app or a CLI. Several template defaults
   turn out to be app-shaped, and a library is the first thing to notice.
2. **It is a live `claude-plus-agents` instance.** `docs/harvests/instances.md`
   records `uw` as *"dead — abandoned, nothing to harvest (was the only
   plus-agents instance)"*, and names the plus-agents blind spot as still open.
   This closes part of it.

The repo-specific half of this — what `irradiance` decided to do about each item
— is `decisions/DEC-000-template-instantiation.md`. What follows is only the
part that generalises to other instances.

---

## 1. `just init` is one-shot, interactive, and unrecoverable-by-design

**What happened.** The wrong variant was picked. Recovery took `git reset --hard`
+ `git clean -fd`, and worked *only because nothing had been committed yet*.

**Why it's structural, not user error.** `justfile:61-62` copies the variant over
the root and then `rm -rf variants/` in the same `&&` chain. The guards at
`justfile:34-45` then refuse to re-run and say *"Restore from git or re-clone."*
That is honest, but it means the single most consequential choice in the
template's life is made:

- at the moment the user knows least about the difference,
- through a bare `read` with no confirmation step (`justfile:50-58`),
- with the undo path outside the tool.

It is also worse than it looks. `justfile:70` offers `fresh-history.sh -y` in the
same run. Answer `y` and the git history the guard tells you to restore from is
the thing that was just rewritten.

**Cheap fixes, in increasing order of effort.**
- Echo the chosen variant and require a confirmation before the destructive step.
- Accept `just init <variant>` non-interactively, so it is scriptable and
  reviewable, and interactive only when the argument is absent.
- Do not `rm -rf variants/`. `git rm -r --cached` + delete, or defer removal to a
  separate `just init-finalize`, keeps the undo inside git where the guard already
  points.

Any one of these removes the whole class. This hits **every instance, exactly
once.**

---

## 2. There is no `just new-project`

**Measured.** `just --list` offers `new-stage`, `new-spec`, `new-spike`,
`new-patch`, `new-handoff`, `new-release-spec`. Not `new-project`.

**The template already knows.** Two scripts carry the workaround in a comment:

- `scripts/new-stage.sh:37-39` — *"A hand-created project (copied from
  project-brief.md) may not have a stages/ dir yet; create it so scaffolding
  works without a separate new-project step."*
- `scripts/new-spec.sh:73-75` — the same, for `specs/`.

So the directories get created as a **side effect** of creating a stage or spec,
but `brief.md` never does. The result is that the one artifact carrying the value
thesis (AGENTS.md §3) — the thing `just status`, `just roadmap` and
`just close-project` all read — is the only artifact in the hierarchy with no
tooling behind it.

**Fix.** `just new-project "title"`: copy `_templates/project-brief.md`,
substitute the next `PROJ-NNN`, `repo.id` and `created_at`, `mkdir -p`
`stages/ specs/done/ handoffs/`. It is the same twenty lines as
`new-stage.sh`, and it deletes both workaround comments.

---

## 3. `version.scheme` defaults to `calver`, which is wrong for every library

**Measured.** Template default: `variants/claude-plus-agents/.repo-context.yaml:60`
→ `scheme: calver`.

The template documents the tradeoff correctly, three lines above the default:
*"semver → for a library/public API whose number must signal compatibility to
consumers."* DEC-007 justifies `calver` as the default because it "just works"
and needs **zero judgment**. That reasoning is sound for an app and inverted for
a library: a library's version number **is** a compatibility claim that consumers
depend on mechanically. `^0.3` means something. `v2026.08.0` means nothing a
resolver can use.

**Why it's worth fixing even though the edit is one line.** The cost isn't the
edit, it's that the wrong default is *silent* — nothing surfaces it, and the
consequence (a published version number that lied) only appears at the first
release, long after it can be cheaply changed.

**Fix options.** Ask at init alongside the variant. Or have `just next-version`
warn once when the repo looks like a library — a `Cargo.toml` with `[lib]` and no
`[[bin]]`, a `package.json` with no `bin`, a `pyproject.toml` with no scripts.

**Meta-finding.** This default survived eight instances because every one of them
was an app or a CLI. Worth treating as sampling bias in the dogfood corpus rather
than as a settled default.

---

## 4. Example content survives init unevenly — and the leftovers are load-bearing

`scripts/scaffold-clean.sh` exists precisely for this, and its header comment
states the principle exactly right: *"None of that describes the app you're about
to build."* But `TEMPLATE_DOCS` (`scaffold-clean.sh:29`) covers **four root
files**, and nothing else. Two seeded artifacts survive:

**The example project.** `projects/PROJ-001-example-mvp/` — 7 files in the
plus-agents variant, including a `HANDOFF-001`. It has to be deleted by hand, and
it occupies **PROJ-001**, the id a real first project wants. Reclaiming the number
is manual.

**The example decision — worse, because it is invisible.**
`decisions/DEC-001-example-structured-logging.md` survives entirely. In this
instance, after full scaffolding:

- `just status` reports **"Total decisions (across all projects): 1"** for a repo
  that has made zero decisions.
- The file says `repo: my-app` (line 17).
- It cites a `use-project-logger` constraint (lines 81 and 103) that this
  instance **deliberately deleted** from `guidance/constraints.yaml`.
- `just decisions-audit` reports **"✓ All 1 decision(s) clean"**.

That last line is the finding. The auditor validates *structure* — id, filename
match, `created_at`, `type`, `status`/`superseded_by` agreement — and never checks
whether a cited constraint exists. A DEC pointing at a deleted constraint is
exactly as green as a correct one.

**Two fixes, both small.**
- Extend `scaffold-clean.sh` to `projects/PROJ-*-example-*/` and
  `decisions/DEC-*-example-*.md`. The `*example*` naming already makes them safe
  to match, and the script already prints what it removed.
- Add a referent check to `decisions-audit`: warn when a DEC's
  `Related constraint:` / `references.constraints` names an id absent from
  `guidance/constraints.yaml`. This is the same class as `--changed`, and it
  catches a real dangling edge that structural linting cannot see.

---

## 5. `constraints.yaml` ships app-shaped placeholders

Template as shipped (`variants/claude-plus-agents/guidance/constraints.yaml`):

- `use-project-logger` (line 29)
- `no-auth-changes-without-approval` (line 37)

Neither has any meaning in a library with no logging framework and no auth. Both
were replaced here with five domain constraints. The app-shape runs into the prose
too: AGENTS.md §2 used *"we use pino for logging"* as its illustration of a
repo-level decision.

Note the coupling to finding 4: the dangling reference there exists **because**
the seeded constraint had a seeded referrer, and the two were cleaned at different
times. Seeded artifacts that cite each other must be removed together or not at
all.

**Fix.** Either mark seeded entries (`seeded: true`) so `just validate` can say
*"3 seeded constraints still present — replace or delete them"* once real work
starts, or keep only the genuinely domain-neutral three (`no-secrets-in-code`,
`test-before-implementation`, `one-spec-per-pr`) and move the app-shaped pair into
a commented `# examples` block.

---

## 6. ✅ WIN — the built-in spike lane beat an externally-designed alternative

**This is the finding worth carrying forward, and it is a template win, not
friction.**

An external plan written before this repo was scaffolded had designed, from
scratch, a **"STAGE-000 spike stage"** to hold the pre-project exploration. The
template already had the spike lane (`docs/decisions/DEC-012-spike-lane.md`), and
the primitive is strictly better than the invention in three specific ways:

1. **A spike attaches to the repo, not a project** — `spikes/` sits at the root.
   So it can precede the project it informs. A STAGE-000 forces the exploration
   *inside* a project that framing has not yet earned, which is backwards: the
   spike exists because the shape isn't known, and a stage presumes the shape.
2. **`test-before-implementation` is explicitly suspended in the spike lane.** A
   stage cannot do that without punching a hole in a blocking constraint. The
   lane makes the exception principled instead of a violation.
3. **`inconclusive` is a named, valid outcome.** A timebox that expires produces
   a *result* rather than pressure to extend. A STAGE-000 has no vocabulary for
   "we asked, and we don't know" — stages ship or they don't.

It works cold, too: with a single spike file present and nothing else, `just
status` surfaced it unprompted — *"Land every spike (just archive-spike) — an
un-landed spike is undocumented decisions."*

**The generalisable lesson is not "the lane is good" — it is that a competent
independent planner reached for the same shape without knowing the lane existed.**
That is strong evidence DEC-012 identified a real primitive. It also means the
invention happened because the lane was **not discoverable at the moment it was
needed** — during planning, before the repo existed. Two candidates:

- Name the spike lane in `GETTING_STARTED.md`'s first page and in
  `FIRST_SESSION_PROMPTS.md`'s project-frame prompt, as the answer to *"what if
  we don't know the shape yet?"* — that is exactly when the invention happens.
- Consider it a `golden-path` signal in `guidance/signals.yaml`. Per that file's
  own bar (*"a wrong paved road is worse than no road… N=3 or it stays a
  preference"*) this is **N=1**. Capture, don't promote.

---

## 7. ~4,800 lines of template-maintainer content ship into every instance

**Measured.** 97 files from the template's top level (everything outside
`variants/`) survive `just init` into the instance. `scaffold-clean.sh` removes
four of them. What remains that describes the **template repo and other people's
repos**, not this app:

| Path | Lines | What it is |
|---|---|---|
| `docs/ROADMAP.md` | 895 | the template's own roadmap |
| `docs/blog/` | 8 files | posts about building the template |
| `docs/sessions/` | 5 files | the template's own hardening sessions |
| `docs/talks/` | 1 file | a talk about the template |
| `docs/harvests/` | 2 files | incl. `instances.md` — a registry of **other people's repos**, with URLs and harvest state |
| `feedback/2026-0*.md` | 4 files | **other instances'** feedback |
| `reports/daily/2026-04-21.md`, `reports/weekly/2026-W17.md` | 2 files | reports on work that never happened here |
| `.github/TEMPLATE_README.md` | 80 | acknowledged in its own text as *"harmless leftover noise in an instance"* |

**~4,825 lines / ~296 KB.**

Two costs, and the second is the one that matters:

- **Cold-read context cost** — already tracked as the open `context-coldread-cost`
  product signal in `guidance/signals.yaml`. This is that signal with a number
  attached.
- **A correctness hazard.** An agent grepping this repo for facts hits 4,800 lines
  about a *different* repo. Concretely, in this instance `grep -rn "id: my-app"`
  returns hits in `docs/sessions/2026-04-20-hardening-report.md` alongside the one
  real hit; `reports/weekly/2026-W17.md` reads as this repo's weekly report and
  is not. AGENTS.md §13 already tells agents to *"verify any claim you are about
  to act on, against the thing it describes"* — this ships 4,800 lines that look
  like exactly such a thing and describe something else.

`scaffold-clean.sh` already has the right principle and the right shape; only its
coverage stops at the root. Extending it one directory down would close this. Note
`docs/decisions/` is a deliberate exception — AGENTS.md cites DEC-003/004/012/013
as live process rationale, so it must stay. Which leads to the last one.

---

## 8. Two DEC namespaces collide by ID, and only one is visible to tooling

An instance ends up with both:

- `/decisions/` — the instance's own, starting at `DEC-001`
- `/docs/decisions/` — the template's own `DEC-001`…`DEC-013`

Both sequences start at 001, so the collision is **guaranteed**, not incidental.
Right now in this repo, `DEC-001` names two different files.

`scripts/_lib.sh:56-64` resolves it for the tooling — *"Root wins when both
exist"* — which means in an instance `just decisions-audit`, `just dash` and
`find_all_decisions` see `/decisions/` **only**. The consequences:

- A spec writing `references.decisions: [DEC-003]` to mean the template's patch
  lane resolves to nothing, or worse, to an instance `DEC-003` that means
  something else entirely. `_lib.sh:758-762` parses that field into bare DEC-IDs
  with no namespace.
- AGENTS.md's own prose has the ambiguity: §17 says *"Decisions: `/decisions/`"*
  while §8 cites DEC-003/DEC-012 meaning the other directory.

The comment at `_lib.sh:50-55` shows the fork has already bitten once — the DEC
schema drifted between the two copies (*"`type: architecture` on 11 of 13 files,
out of the enum the auditor would have caught"*).

**Fix.** Give the template's own records a distinct prefix (`TPL-003`) or a
distinct directory (`docs/template-decisions/`). Either makes
`references.decisions` unambiguous forever, and neither costs anything but a
rename. This instance works around it in prose (AGENTS.md §10) — a workaround is
the wrong layer for something a rename fixes.

---

## 9. The `affected_scope` hygiene warning is a false positive for root-level files

**Measured.** `DEC-000` in this repo declares `affected_scope: [AGENTS.md,
.repo-context.yaml, guidance/constraints.yaml, decisions/**, feedback/**]`.
`just decisions-audit` warns on the first two:

```
⚠ DEC-000: affected_scope 'AGENTS.md' has no path separator or wildcard —
  it likely matches nothing (did you mean 'dir/AGENTS.md' or '**/AGENTS.md'?)
```

They match fine. `just decisions-audit --changed` on the same tree reports:

```
⚠ DEC-000 — How the spec-driven template was instantiated as `irradiance`
      re-read this decision before committing; your change touches:
        AGENTS.md
```

**Why.** The check at `scripts/decisions-audit.sh:393-403` warns on any glob with
no `/`, `*` or `?`. But the matcher (`decisions-audit.sh:224-231`) regex-matches
globs against **repo-relative** paths, where a root-level file *is* a bare name.
The rationale comment names the motivating case — *"zany harvest: `['_headers']`
never matched the real `public/_headers`"* — which is a **nested** file. Root-level
files were not considered, and they are exactly what a process-scoped decision
governs: `AGENTS.md`, `justfile`, `.repo-context.yaml`, `Cargo.toml`,
`package.json`.

**Cost.** The warning is permanent and unfixable-without-breaking-something: the
suggested `**/AGENTS.md` is a *worse* glob (it would also match a nested
`variants/*/AGENTS.md`), and the correct glob warns forever. Any instance whose
first real decision governs its process files starts with a permanently dirty
audit — which trains people to ignore audit output, the one thing an audit
cannot survive.

**Fix.** Suppress the warning when the bare name matches a file that exists at the
repo root — one `[ -e "${REPO_ROOT}/${g}" ]` test. That keeps every bit of the
original teeth (a bare `_headers` that exists nowhere at root still warns) and
drops the false positive entirely.

---

## 10. `frame-stage` slugifies the whole summary, so a long outline line kills it

**Measured.** A backlog outline whose summary ran ~290 characters produced:

```
cp: .../SPEC-001-not-yet-written-crate-scaffold-cargo-toml-edition-2021-a-measured
-msrv-the-panic-free-clippy-lint-set-forbid-unsafe-code-and-the-rust-ci-jobs-...md:
File name too long
error: recipe `frame-stage` failed on line 118 with exit code 1
```

`scripts/frame-stage.sh:73` computes `SLUGS+=("$(slugify "$sm")")` over the
entire summary with no length cap, and every filesystem in common use caps a path
component at 255 bytes.

**Credit where due: the failure was atomic.** Nothing was written — no partial
specs, no half-edited stage file — so a retry after shortening the line worked
cleanly. That is better behaviour than most tools manage.

**Why it bites.** The stage template invites detail on the backlog line, and the
detail is genuinely useful at framing time — that is where scope and constraints
belong. Nothing warns that the line doubles as a filename. The workaround is to
keep summaries short and move detail into `## Design Notes`, which is arguably
better practice anyway, but it is discovered by crashing into it.

**Fix.** Truncate the slug (say 60 chars) before building the path. One `cut`.
The ID is the identity; the slug is a human hint and does not need to be lossless.

---

## 11. `frame-stage` leaves `(not yet written)` in every filename it creates

**Measured.** Framing five outlines produced ten files all carrying the
placeholder:

```
SPEC-001-not-yet-written-crate-scaffold-cargo-toml-measured-msrv-panic-free-lints-rust-ci.md
SPEC-001-not-yet-written-crate-scaffold-...-timeline.md
```

…and the same string in each spec's H1 (`# SPEC-001: (not yet written) Crate
scaffold: …`).

**The tell that it is a bug, not a convention:** the *backlog lines the same run
rewrites* come out clean —

```
- [ ] SPEC-001 (frame) [S] Crate scaffold: Cargo.toml, measured MSRV, panic-free lints, Rust CI
```

So the placeholder is stripped for the rewritten bullet but not before the slug
and title are derived. One code path strips it, the other doesn't.

**Why it matters more than cosmetics.** These filenames are permanent and get
cited in commits, PRs and handoffs for the life of the project. Every spec in
every instance is born asserting it has not been written — including, confusingly,
after it ships. Renaming afterwards works (verified: `just validate`,
`just status` and `just ready` all resolve specs by `SPEC-NNN` and were unaffected)
but every instance has to discover and do it.

**Fix.** Strip `(not yet written)` in `parse_bullet` alongside the `[S]`
complexity marker at `scripts/frame-stage.sh:60`, so slug, title and bullet all
derive from the same cleaned summary.

---

## Priority (this instance's assessment)

| # | Finding | Severity | Fix cost |
|---|---|---|---|
| 1 | `just init` one-shot + destructive | **high** — unrecoverable, hits everyone once | low |
| 4 | example DEC survives; audit reports it clean | **high** — silently wrong data, dangling ref | low |
| 8 | two DEC namespaces collide by ID | **high** — corrupts `references.decisions` | low (rename) |
| 7 | 4.8k lines of maintainer content per instance | medium — context cost + wrong-repo facts | low |
| 3 | `calver` default wrong for libraries | medium — silent until first release | trivial |
| 2 | no `just new-project` | medium — the value-thesis artifact is untooled | low |
| 5 | app-shaped seed constraints | low — obvious on read | trivial |
| 9 | `affected_scope` warns on correct root-level globs | low — but it trains people to ignore the audit | trivial |
| 10 | `frame-stage` slugs uncapped → ENAMETOOLONG | medium — hard stop mid-framing (but atomic) | trivial |
| 11 | `frame-stage` bakes `(not yet written)` into every filename | medium — permanent, cited for the project's life | trivial |
| 6 | **spike lane beat the external design** | **win** — capture, don't promote (N=1) | n/a |

Findings 1, 4, 7, 8 and 9 are all the *same shape*: the template knows the right
principle, states it in a comment, and applies it to a scope the principle does not
quite fit — too narrow in 1/4/7/8, too broad in 9. Adjusting the reach of an
existing mechanism would close all five; none needs a new one.

## Resolution

Open. Repo-specific dispositions are in
`decisions/DEC-000-template-instantiation.md`.
