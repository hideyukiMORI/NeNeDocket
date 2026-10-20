# NeNe Docket — Agent Guide

Short English entry point. The authoritative handbook is [CLAUDE.md](CLAUDE.md) (Japanese);
the normative documents live in [docs/](docs/).

## Project identity

- Product: NeNe Docket — a small, fast issue tracker for Windows and X11. A docket is a folder; put it on a network share and a team edits it together with no server
- Language / UI: Rust, edition 2024 / none (Win32 + Direct2D on Windows, plain X11 through `x11rb` + cairo `ImageSurface` on Linux; every widget is drawn by hand)
- Data: one append-only journal per account, merged deterministically by a pure function in core. Undo appends the inverse op. Nothing is ever rewritten ([ADR 0004](docs/adr/0004-append-only-journal-per-account.md))
- Workspace: `crates/core` + `xtask` today; planned `crates/{application,store,adapters-x11,adapters-windows,ui-x11,ui-win32,app}`
- Governing principle: **one meaning, one canonical implementation path, enforced by machines**

## Status

Issue #1 (2026-10-21) put the documents in place. Issue #2 ported nene-ruler's machinery (`[workspace.lints]`, `clippy.toml`, `deny.toml`, `xtask`, `Makefile`, CI, hooks)
and measured it here: `make prove` (26 real-tool proofs) and `cargo test -p xtask` (80). The rules that fired are `active` in `docs/QUALITY_GATES.md`; the rest stay `planned`.
Issue #4 added `crates/core` with the widget kernel (immediate mode, ADR 0006): layout, hit testing, focus, text editing state, draw commands. **No UI, store or application crate yet.**

## Required reading before changing production code

0. `SPECIFICATION.md`
1. `docs/ARCHITECTURE_CONSTITUTION.md`
2. `docs/PROJECT_LAYOUT.md`
3. `docs/CODING_RULES.md`
4. `docs/QUALITY_GATES.md`
5. `docs/DEVELOPMENT_WORKFLOW.md`
6. `docs/COMMIT_CONVENTIONS.md`
7. `docs/GLOSSARY.md`

Then the active issue and the relevant accepted ADRs.

## Agent rules

- Do not invent a second implementation path because it is locally convenient.
- Do not weaken a gate to make a change pass. Fix the code instead. `forbid` lints cannot be suppressed; `deny` lints only with `#[expect(lint, reason = "<rule id>: ...")]`.
- Do not mark a rule `active` in `docs/QUALITY_GATES.md` before its enforcement exists and `docs/quality/gate-proofs.md` has its row.
- Do not read the current time, randomness, environment, files, or threads outside `crates/adapters-x11`, `crates/adapters-windows` and `crates/store`. Core crates are `#![no_std]`.
- Do not rewrite, truncate or reorder a journal. Append only. Each process writes only its own account's journal.
- Do not put merging, ordering or state transitions in the UI. Core computes layout and hit regions; the UI paints and emits intents.
- Do not write `_` in a `match` over a closed enum; it disables exhaustiveness checking.
- Do not add dependencies without an ADR and a measured `deny.toml` update. Core, application, `store` and `xtask` have zero dependencies.
- Do not claim a command passed unless it was actually executed.
- Prefer the smallest change that fully follows the canonical path.

## Verification

Pick the narrowest checks the diff needs (`make lint`, `make test`, `make conformance`, `make deny`, `make doc-check`).
Any change to `merge` must run the test that reads the same ops in a different order and expects the same state.
`make check REASON='...'` runs everything and refuses an empty reason. `make ci` is what the required GitHub check runs.
`make prove` re-measures that the gates fire; run it whenever a gate changes.

## Required completion report

Issue and rule IDs, files and behavior changed, verification commands and results (and where the record lives),
documentation or schema changes, `#[expect]` suppressions added (or `none`), remaining risks.

Investigation-only requests do not authorize editing, committing, pushing, or opening PRs.
