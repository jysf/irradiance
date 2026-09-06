---
source: "irradiance (PROJ-001) — jysf"
captured_at: 2026-09-06
captured_by: claude
status: open
---

# `just new-*` mints IDs from the working tree, so two branches mint the same one

## The issue

In the maintainer's words:

> "file the ID minter feedback, as that has happened before, maybe we need to
> create an index or catalog of IDs that get created and can be updated even if
> the file isn't checked in"

`scripts/new-spec.sh`, `new-stage.sh`, `new-handoff.sh` and `new-patch.sh` assign
the next id by scanning the **working tree** for existing files. A tree only ever
shows the branch it is on, so an id minted while another branch is in flight
collides silently.

**N=2, and both were caught by a human noticing — not by tooling.**

| date | what collided | how it was caught |
|---|---|---|
| 2026-08-22 | `SPEC-010` framed on `feat/spec-005-metadata-oracle`, then `just new-spec` assigned `SPEC-010` **again** on `main` for a different spec | the same session had both in view |
| 2026-09-06 | `HANDOFF-038` written and pushed on `feat/spec-016`, then `just new-handoff PATCH-002 verify` assigned `HANDOFF-038` **again** on `fix/patch-002` | the orchestrator had created the other an hour earlier and recognised the number |

⚠ **The defect is invisible to history by construction, and that is the strongest
argument for the maintainer's proposal.** A
`git log --all --diff-filter=A --name-only` sweep across this repo's entire
history returns **zero** collisions — because every occurrence so far was caught
in-session and renamed *before* the duplicate was ever committed. So:

- the true N is not 2, it is **"twice that anyone noticed"**;
- no audit of committed files can ever find an instance;
- and the failure mode when it *isn't* caught is a merge producing two files with
  the same id and different slugs, in a repo whose §2 says **"IDs are globally
  unique and continuous across the repo"** and whose §15 rests on ids surviving
  handoff boundaries.

Both artifacts also look correct in isolation. `ls handoffs/` on either branch
alone shows one `HANDOFF-038`; only a cross-branch view shows two.

## Context

Reported at the end of a session that had four branches in flight at once
(`spec-014`, `spec-015`, `spec-016`, `patch-002`) — the working mode this
template's `claude-plus-agents` variant actively encourages, since §13 tells
concurrent sessions to use separate worktrees. **The more the variant succeeds,
the more often this fires.**

## Priority (their assessment)

Worth fixing, and the maintainer proposed the shape: *an index or catalog of IDs
that get created and can be updated even if the file isn't checked in.*

That is the right shape, and it is worth saying why in template terms:

- **Minting must write, not just read.** The bug is that minting is a pure read
  of a mutable, branch-local surface. An append-only index turns it into a claim.
- **The index has to be updatable without a commit**, which is the maintainer's
  own qualifier and the crux: an index that only counts committed files
  reproduces the bug one level up.
- **A collision should be loud at mint time**, not at merge time. `new-*.sh`
  already `die`s with its own message elsewhere; this is that pattern.

Sketches, not a design — the template owner should choose:

1. **Append-only ledger** (`.ids/ledger.tsv` or similar), one line per mint:
   `id, kind, slug, branch, timestamp`. Committed, but **appended at mint** so it
   travels with the branch, and a merge conflict on the append is itself the
   collision signal — a conflict is a *good* failure here.
2. **Mint from `git log --all` rather than the tree**, so other branches' ids are
   visible without an index. Cheaper, no new file, but only sees *fetched* refs
   and nothing uncommitted — weaker than (1), and it would not have caught either
   instance above, since both were uncommitted at mint time.
3. **A `just id-audit`** that fails when two artifacts across all refs share an
   id. Detection, not prevention; useful alongside either of the above, useless
   alone for the reason in the table — nothing reaches history to audit.

⚠ (2) is the tempting one and it is the weakest. Worth stating explicitly so it
is not chosen by default.

## Resolution

Open. Tracked in this repo as `guidance/signals.yaml`
→ `spec-ids-collide-across-unmerged-branches` (`bar: 2`, `status: open`,
`disposition_at: project-close`), which carries both instances in full.

This is **template-level friction**, not app-level: the minter scripts are the
template's (`scripts/new-*.sh`), the ids-are-globally-unique rule is the
template's (§2), and every variant that runs concurrent branches inherits both.
Filed here per `DEC-000` and AGENTS.md §15's *"Template-level friction also goes
to `/feedback/`"*.
