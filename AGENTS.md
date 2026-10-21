# NeNe Docket — Agent Guide

Short English entry point. The authoritative handbook is [CLAUDE.md](CLAUDE.md) (Japanese);
the normative documents live in [docs/](docs/).

## Project identity

- Product: NeNe Docket — a small, fast issue tracker for Windows and X11. A docket is a folder; put it on a network share and a team edits it together with no server
- Language / UI: Rust, edition 2024 / none (Win32 + Direct2D on Windows, plain X11 through `x11rb` + cairo `ImageSurface` on Linux; every widget is drawn by hand)
- Data: one append-only journal per account, merged deterministically by a pure function in core. Undo appends the inverse op. Nothing is ever rewritten ([ADR 0004](docs/adr/0004-append-only-journal-per-account.md))
- Workspace: `crates/{core,application,store,adapters-x11,adapters-windows,ui-x11,ui-win32,app}` + `xtask`
- Governing principle: **one meaning, one canonical implementation path, enforced by machines**

## Status

Issue #1 (2026-10-21) put the documents in place. Issue #2 ported nene-ruler's machinery (`[workspace.lints]`, `clippy.toml`, `deny.toml`, `xtask`, `Makefile`, CI, hooks)
and measured it here: `make prove` (26 real-tool proofs) and `cargo test -p xtask` (80). The rules that fired are `active` in `docs/QUALITY_GATES.md`; the rest stay `planned`.
Issue #4 added `crates/core` with the widget kernel (immediate mode, ADR 0006). Issue #5 added `crates/application` (the screen state owner), `crates/ui-x11` (x11rb + cairo) and `crates/app`; the window runs on Xvfb with a 10,000-row table, search, filter and scrolling. Japanese is drawn by a font family chosen at startup (ADR 0007). Issue #7 added the journal core to `core`: `Op`/`OpId`/`ItemId`, `merge` (a pure function; the same set of ops in any order gives the same `Docket`) and `history`. Issue #9 added `crates/store`: journal v1 (one op per line with a CRC-32, append + fsync, incremental re-reads of other accounts' journals, `docket.v1.txt`). Issue #11 wired the screen to the journal: `Ledger` (merged docket, Lamport clock, undo stack) and `Ports` in application, `FileJournal` in store, and `crates/adapters-x11` (system clock, account file under `$XDG_CONFIG_HOME`, a refresh tick thread that sends an X `ClientMessage` every 3 s; ADR 0008). `nene-docket <folder>` opens a docket; add, title, assignee, due date, labels, state, delete and Ctrl+Z / Ctrl+Y append ops, and two processes on the same folder see each other's changes within seconds (Xvfb, gate-proofs §14). Issue #16 added the settings file (`settings.v1.txt`: recent dockets and the refresh interval; `nene-docket` without arguments reopens the latest docket). Issue #18 added the history pane (`History` checkbox: when, who, what and from-what-to-what for the selected issue, undo rows muted). Issue #22 added `crates/adapters-windows` and `crates/ui-win32` (Win32 + Direct2D / DirectWrite, `SetTimer` refresh, `%APPDATA%` files; ADR 0009): `make lint-windows` is clean and `make build-windows` produces the exe, **but it has never been run on Windows**. Issue #27 added six UI languages (ja / en / zh-hans / zh-hant / ko / es; one wording table in core, ADR 0011) and the 125 built-in colour schemes ported from nene-nib (derived from three colours, ADR 0012); settings are now v2 (`settings.v2.txt`, v1 is read and migrated). Issue #29 accepted ADR 0010 and added signatures and permissions: `ed25519-dalek` in core (its only external crate), a secret key in `account.v2.txt` (v1 is migrated), public keys and names under the docket's `keys/`, an owner-signed `permissions.v1.txt` edited from the command line (`--permissions on|off`, `--allow`, `--disallow`, `--group`, `--ungroup`), and journal v2 lines (signed) in dockets with permissions on; readers verify every line and drop and count the ones that fail. History shows account names from `keys/`. Issue #31 added the multi-line body: a `Body` checkbox opens a pane under the table (`text_area` in core; Enter inserts a line break, Up/Down walk lines, the view follows the caret line), the search matches the body, and history shows line breaks as `↵`. Issue #33 added the members pane (`Members` checkbox: name, id, role from `keys/` and the permissions); the owner turns permissions on/off and toggles editors by clicking rows. The initial window is now 1120 px wide. Issue #35 made history rows clickable (re-picks that value as a new `Set`, FR-008) and shows a notice when a field commit overwrote a change that arrived while editing. Issue #36 gave the body and members panes scroll bars (the body follows the caret line while focused). Issue #39 shows the item as it was at any history line while the pointer rests on that line (FR-006; display only). Issue #41 made table columns resizable by dragging header borders; settings are now v3 (`columns = …`, v2 and v1 are read). Issue #43 made columns reorderable by dragging headings; settings are v4 (`order = …`). Issue #48 reads the X11 appearance from the GTK conventions (`GTK_THEME=…:dark` or `gtk-3.0/settings.ini`), so `system` can be dark on Linux. Issue #51 accepted ADR 0013 (one key per machine; a second machine is a second account): `permissions.v1.txt` gained `succeeds = NEW: OLD` lines (`--succeed NEW OLD`; the old id's history shows the new name and the old id leaves the members pane) and a `handover` line (`--owner ACCOUNT`; the previous owner signs the handover and the new owner re-signs on first open; an owner who lost the key is replaced by hand-editing `docket.v1.txt`). Issue #53: clicking a member row now selects it, and an edit row (an `Editor` checkbox and a comma-separated `Groups` field) appears above the pane for the owner. Issue #54 added a `No successor` select box to that edit row: choosing another member writes `succeeds = NEW: OLD` and the successor takes over the old id's editor entry and groups. Issue #55 made the border between the table and the lower panes draggable; settings are v5 (`pane_px = auto|<px>`, v4 and older are read). Issue #56 accepted ADR 0014 (audit): every permission change is also appended to the owner's journal as an `admin` line (`OpKind::Admin(AdminCommand)`, `AdminCommand` moved to core; merge ignores it, undo skips it), the history pane lists the whole docket chronologically when no issue is selected, and `nene-docket <folder> --audit` prints the same timeline as tab-separated text. Issue #61 (ADR 0015) raises FR-017 to 100,000 issues / 3,000,000 ops: stage 1 caches the visible-row list per (ledger generation, search, filter, show-done) and applies later ops in place in `Ledger::absorb` (100k rows: 22 ms → 1.4 ms per frame); stage 2 writes a per-account snapshot (`snapshot/<account>.v1.txt` + a 32-byte-record `.idx` per-item index) every 100 refreshes and on close, resumes from it on the next open, and no longer keeps ops in memory (history, undo and the audit read journal lines through `JournalPort::ops_of` / `recent_ops` / `all_ops`): 3,000,000 ops now open in 0.9 s at 178 MB instead of 8.5 s at 703 MB. Issue #62 accepted ADR 0016 (attachments): a file is copied into the docket folder as `attachments/<creator>:<seq>/<n>-<name>` (temp name + rename, never deleted) and an `attach` line (name, size, SHA-256 — hand-written in core, no new crate) or a `detach` line is appended; the `Attachments` checkbox opens a pane under the table (path field + `Attach` / `Detach`, altered copies are marked `⚠`), `--attach <creator>:<seq> <path>` does the same from the command line, undo re-attaches, and history / `--audit` show `attached: name` / `detached: name`. Viewers and drag-and-drop are Issues #63–#66. **No IME preedit; translations and derived palettes are not yet reviewed by the owner.**

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
- Do not add dependencies without an ADR and a measured `deny.toml` update. Core has exactly one external crate (`ed25519-dalek`, ADR 0010); application, `store` and `xtask` have zero.
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
