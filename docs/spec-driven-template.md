# The spec-driven template

*This is the process documentation for the spec-driven multi-agent workflow
this repo is built on. It is the template's own README, kept here as a
reference after the root `README.md` was made specific to this app
(irradiance). For what this repo actually builds, see the root
[`README.md`](../README.md). For the full arc — shaping an idea through to a
shipped wave — see [`PLAYBOOK.md`](PLAYBOOK.md).*

The repo uses a spec-driven multi-agent workflow. Claude handles architecture and review; a separate implementer agent (Kilo Code, Factory Droids, AdaL, Cursor, etc.) handles implementation.

## Hierarchy

```
Repo (this app)
 └─ Project (a wave of work: "MVP", "v2 improvements")
     └─ Stage (a coherent chunk within a project)
         └─ Spec (an individual task)
              └─ Cycle (Frame → Design → Build → Verify → Ship)
```

## Getting started

**First time?** Read `GETTING_STARTED.md` — it walks you through your first project end-to-end.

**Daily work?** Run `just --list` to see available commands.

**Common commands:**
```bash
just status                        # See active project, stage, specs by cycle
just backlog                       # Spec-grained: what's next in the active stage
just roadmap                       # Stage-grained: where this project is going
just new-spec "title" STAGE-001    # Scaffold a new spec
just advance-cycle SPEC-001 verify # Update a spec's cycle
just archive-spec SPEC-001         # Move a shipped spec to done/
just review                        # Print the weekly review prompt
just report daily                  # Generate today's daily report
just report weekly                 # Generate this week's weekly report
just report status                 # Snapshot `just status` to reports/daily/<date>-status.md
```
`report-daily` / `report-weekly` remain as permanent aliases for
`report daily` / `report weekly`.

## Reports

`just report-daily` and `just report-weekly` generate quantitative
snapshots under `reports/daily/` and `reports/weekly/` from spec
front-matter and git log. Daily reports show specs by cycle, value
thesis, cost activity today, and flags. Weekly reports aggregate
ships, cycle times, cost by cycle and interface, and value
advancement. Reports are stand-alone artifacts — re-running
overwrites, so they're always a current snapshot.

## The app itself

The workflow above is the *meta-process*; every repo built on this template
also needs a root `README.md` about the *app*. That is what the root
[`README.md`](../README.md) is for in this repo. When you scaffold a new app
from this template, its root README should cover:

- What the app does (1 paragraph)
- How to run it locally (link to `AGENTS.md` Section 6, "Commands (exact)")
- How to run tests

## Where things live

| Path | Purpose |
|---|---|
| `AGENTS.md` | Conventions for agents working in this repo |
| `.repo-context.yaml` | Structured metadata about the app |
| `docs/` | Architecture, data model, API contract |
| `guidance/` | Repo-level rules, open questions, and the signals ledger (`just dash signals`) |
| `decisions/` | Decision log (accumulates across projects) |
| `projects/` | Each project (wave of work) lives here |
| `projects/*/brief.md` | What this project is and why |
| `projects/*/stages/` | Stages within a project |
| `projects/*/specs/` | Specs within a project |
| `projects/*/handoffs/` | Architect → implementer delegation records |
| `src/` | The actual app code |

## License

Each app sets its own license. This repo is dual-licensed under
MIT OR Apache-2.0 — see [`LICENSE-MIT`](../LICENSE-MIT) and
[`LICENSE-APACHE`](../LICENSE-APACHE).
