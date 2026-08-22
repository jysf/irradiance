# app.just — YOUR project's commands (build, dev, test, deploy, …)
#
# This file is PROJECT-OWNED. The root `justfile` is template-managed and
# imports this one (`import? 'app.just'`), so:
#   - Put every app-specific recipe HERE, not in the root justfile. Template
#     updates then never conflict with your commands.
#   - Everything defined here is a normal `just` recipe: `just build`,
#     `just dev`, `just test`, etc. Run `just --list` to see them alongside
#     the template's workflow commands (`status`, `new-spec`, …).
#   - These should mirror the "Commands (exact)" block in AGENTS.md §6 — that
#     section documents them for humans; these make them runnable.
#
# Filled in by SPEC-001 (crate scaffold). `irradiance` is a Rust library —
# no dev server, no install step beyond the toolchain itself.

# Install dependencies — nothing to install; the toolchain is the dependency
install:
    cargo fetch

# Run the app locally — no dev server; this is a library. Watch-build instead.
dev:
    cargo build

# Build the app / produce the release artifact
build:
    cargo build --release

# Run the app's test suite.
#
# `corpus-status` runs FIRST and prints one line per manifest entry — present,
# or SKIP + the path it looked at. That is the whole point of the ordering:
# measured at SPEC-002 design, an `eprintln!` inside a PASSING test is captured
# (`cargo test` -> 0 SKIP lines; `cargo test -- --nocapture` -> all of them), so
# a skip announced from inside the harness is silent in exactly the case it
# exists for. A silent skip reports green for work it never did — the same
# defect class as an oracle that cannot go red (DEC-003, AGENTS.md §12 bar 4).
#
# Do NOT "fix" this by adding `-- --nocapture` below. That buries the corpus
# signal in full test output instead of surfacing it, and this recipe's
# contract is that `just test 2>&1 | grep SKIP` names any absent file with no
# extra flags.
test:
    cargo run --quiet --all-features --example corpus-status
    cargo test --all-features

# Run ONLY the live metadata oracle (SPEC-005) — exiftool/dnglab cross-checks
# plus the red-proof. Needs $IRRADIANCE_CORPUS_DIR for the tier-B half and
# exiftool/dnglab on PATH; both halves skip loudly, naming what's missing,
# when absent. The tier-A red-proof (`oracle_is_clean_on_an_unmodified_reading`,
# `oracle_names_the_one_field_that_was_perturbed`) needs neither and is the
# only half CI can run.
oracle-meta:
    cargo test --all-features --test metadata_oracle

# Lint / format check
lint:
    cargo clippy --all-targets --all-features -- -D warnings
    cargo fmt --check

# Type check — Rust has no separate typecheck; `check` is the fast path
typecheck:
    cargo check --all-targets --all-features

# Licences — the permissive-only gate (constraint no-copyleft-dependencies).
#
# ⚠ This covers the LIBRARY's graph ONLY. `fuzz/` is a separate package
# (DEC-011) and cargo-deny evaluates one manifest's graph at a time, so this
# recipe does not see a single one of libfuzzer-sys's dependencies. Run
# `just deny-fuzz` too — see below.
#
# Licences — the permissive-only gate over the LIBRARY's graph.
deny:
    cargo deny check licenses

# The same gate over the FUZZ graph (`irradiance-fuzz`, DEC-011). The tenth gate.
#
# ⚠ Not redundant with `just deny`, and the difference is not cosmetic: the two
# graphs share nothing except this library. DEC-011 originally recorded this as
# a corner cargo-deny could not reach and hand-checked the licences instead;
# SPEC-003's verify cycle ran it and it FAILED on two real defects the
# hand-check had missed (libfuzzer-sys declares `(MIT OR Apache-2.0) AND NCSA`
# — conjunctive — and `irradiance-fuzz` carried no `license` field at all). The
# gate was never absent; it was never invoked.
#
# `--manifest-path` is the whole fix. Keep both recipes green before claiming
# the licence gate passes.
#
# Licences — the same gate over the FUZZ graph.
deny-fuzz:
    cargo deny --manifest-path fuzz/Cargo.toml check licenses

# Prove the LIBRARY's panic-free lint policy actually rejects violating code —
# one violation per policy lint, injected into a temp-dir copy of src/lib.rs,
# so src/lib.rs's own #![deny(...)] is what must reject them. The UNMUTATED
# copy is run first as a negative control and must pass; without that, a red
# proves only that clippy failed, not that the policy rejected anything
# (constraint oracle-must-be-shown-red applied to a gate; SPEC-001 acceptance
# criterion 5; DEC-009, superseding DEC-007).
# See scripts/lint-red-proof.sh.
lint-red-proof:
    ./scripts/lint-red-proof.sh

# Close the #[allow] escape hatch on the LIBRARY (SPEC-006).
#
# `just lint-red-proof` pins the crate-root #![deny(...)] policy; it cannot see
# an #[allow] BENEATH that root, and one such attribute on a pub fn shipped two
# panics past seven green gates. `-F` is `--forbid`: a forbidden lint cannot be
# re-allowed in source, and attempting it is compiler error E0453 — so this
# fires on the escape hatch itself, whether or not the code beneath it panics.
# No text matching, which is the point (attribute text lives inside src/lib.rs's
# own module docs).
#
# Scope is `--lib` deliberately: it excludes #[cfg(test)] modules (not compiled
# without cfg(test)) and src/bin/irr.rs (a different target), so the sanctioned
# exceptions need no per-site special-casing. `--all-targets` here exits 101 on
# the honest tree, because the test module legitimately allows.
lint-no-allow:
    cargo clippy --lib --quiet -- \
        -F clippy::unwrap_used -F clippy::expect_used -F clippy::indexing_slicing \
        -F clippy::panic -F clippy::arithmetic_side_effects

# lint-ci — clippy AS CI SEES IT. `cargo clippy` on PATH is Homebrew's 0.1.97;
#   CI uses dtolnay/rust-toolchain@stable, which FLOATS and is 0.1.98 today.
#   Every lint clippy adds is a new CI failure under `-D warnings`, and a green
#   local `just lint` says nothing about it. That gap ran CI red for FOUR
#   shipped specs (PATCH-001) while every verify cycle reported "ten gates
#   green" — locally.
#   ⚠ THE PATH= PREFIX IS LOAD-BEARING and is the FOURTH instance of the
#   `+toolchain` trap family (guidance/toolchain-brief.md). `~/.cargo/bin/cargo
#   +stable clippy` ALONE reports clippy 0.1.97: the outer command resolves via
#   the shim but `cargo-clippy`/`clippy-driver` are then found on PATH, where
#   Homebrew's win. With the prefix it reports 0.1.98. Measured 2026-08-22.
#   Run this before every push. It is the only local command that can see what
#   CI will see.
lint-ci:
    PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +stable clippy \
        --all-targets --all-features -- -D warnings

# Run the IFD fuzz target (SPEC-003, AGENTS.md §12 bar 2).
#
# ⚠ The `PATH=` prefix is load-bearing and is NOT redundant with the
# `~/.cargo/bin/cargo` below. `cargo fuzz` shells out to a bare `"cargo"
# "build"`, and that INNER call resolves to Homebrew's stable cargo, which
# rejects `-Zsanitizer=address` ("error: 1 nightly option were parsed").
# Invoking the shim as the OUTER command does not fix the inner one. See
# guidance/toolchain-brief.md, "The SECOND `+toolchain` trap" — measured
# 2026-08-18 and re-confirmed 2026-08-20.
#
# Two corpus dirs on purpose: `fuzz/seeds/ifd` is the committed hand-built
# tier-A seed set (`just fuzz-seeds` regenerates it), `fuzz/corpus/ifd` is
# where libFuzzer writes what it finds and is gitignored, so a run leaves
# `git status` clean.
fuzz seconds="60":
    mkdir -p fuzz/corpus/ifd
    PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
        fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time={{seconds}}

# Regenerate the committed fuzz seed corpus from tests/support/tiff.rs, so a
# fixture added to the hostile-input test lands in the fuzz lane too.
fuzz-seeds:
    cargo run --quiet --all-features --example fuzz-seeds

# The MSRV gate (rust-version = "1.90" in Cargo.toml).
#
# ⚠ The `~/.cargo/bin/` prefix is load-bearing — this is the THIRD instance of
# the `+toolchain` trap and the reason this recipe exists at all. A bare
# `cargo +1.90.0 check` fails with `error: no such command: +1.90.0`, because
# `cargo` on PATH is Homebrew's real cargo, which does not understand
# `+toolchain` syntax at all. Only the rustup shim does. See
# guidance/toolchain-brief.md, "The THIRD `+toolchain` trap".
#
# Until SPEC-003's punch-list round this was the ONE gate of the ten with no
# recipe, so it was the one that handed you the raw failing command — measured
# twice, by two different agents, each losing a loop to it.
#
# A pin, not a floor: `rust-version` is only meaningful if something compiles
# against EXACTLY that toolchain. Do not swap this for `+stable`, and do not
# bump the pin without compiling against an actually-older toolchain first
# (SPEC-001 acceptance criterion 2).
#
# MSRV — compile the whole target set against exactly 1.90.0.
msrv:
    ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features
