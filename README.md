# NeNe Docket

A small, fast issue tracker for Windows and X11. Rust, no UI toolkit, no server.

> **Status (2026-10-21):** `nene-docket <folder>` opens a docket on X11. Issues are added, renamed, moved between states and deleted in a hand-drawn table (`x11rb` + cairo, under 1 ms per frame at 10,000 rows); every edit is appended to this account's journal and fsynced, Ctrl+Z / Ctrl+Y append the inverse op, the `History` pane shows who changed what and when, the UI speaks six languages (ja / en / zh-hans / zh-hant / ko / es) and ships the 125 colour schemes of nene-nib, and two processes on the same folder see each other's changes within 3 seconds (verified on Xvfb). A Windows build (`make build-windows`, Win32 + Direct2D) compiles but has not been run on Windows yet; no IME preedit. Permissions are optional: with them on, every line is signed (Ed25519) and verified by every reader. Rules and gates are measured (`make prove`, 26 proofs; core + application at 98.7% region coverage).

```bash
cargo build --release -p nene-docket
target/release/nene-docket /path/to/docket   # the folder is created if missing
target/release/nene-docket                   # reopens the most recent docket (or ./docket the first time)
```

The account (a random 128-bit id, your `USER` name and a secret key) is created once in `$XDG_CONFIG_HOME/nene-docket/account.v2.txt`; recent dockets, the refresh interval, language, theme and column layout live next to it in `settings.v4.txt`. The config folder belongs to one machine: do not copy it to another (a copied key is reported as a key mismatch). Use a second machine as a second account, and if a key is lost let the docket owner run `--succeed NEW OLD` so the old id's history carries the new name (ADR 0013).

## What it will be

- Open a folder and its issues appear in a table. Edit in place, undo with Ctrl+Z, browse the history of every issue.
- Put the folder on a network share and a whole team edits it at once. Each account appends to its own journal; readers merge all journals deterministically, so nothing is lost and no file lock is needed.
- Crash-safe: every operation is appended within seconds, so at most a few seconds of work can be lost.
- Permissions and groups are optional. Without them, everyone edits.
- Every widget (table, select box, text field, scrollbar) is drawn by the app itself: Win32 + Direct2D on Windows, `x11rb` + cairo on Linux. No Electron, no GTK, no Qt, no JVM.

The requirements are in [SPECIFICATION.md](SPECIFICATION.md) (Japanese).

## How it is built

This repository follows the AYANE strict policy: **one meaning, one canonical implementation path, enforced by machines.**
It is the fifth application of that policy and the third in Rust (after xi-tools and [nene-ruler](https://github.com/hideyukiMORI/nene-ruler), whose documents and machinery this repository inherits).

- `docs/ARCHITECTURE_CONSTITUTION.md` — the rules (ARC-NNN) and how far Rust reaches for each of them
- `docs/CODING_RULES.md` — the approved subset of Rust (RS-NNN); suppressions are two-tiered (`forbid` cannot be overridden, `deny` needs `#[expect(..., reason = "<rule>: ...")]`)
- `docs/QUALITY_GATES.md` — what is enforced right now (active / planned / impossible), the only source of truth
- `docs/quality/gate-proofs.md` — real tool output for every gate that claims to be active
- `docs/adr/` — decisions, including why there is no toolkit and why the data model is append-only journals per account

Verification is chosen per change (`make lint`, `make test`, `make conformance`, ...). The full run needs a reason:

```bash
make check REASON='why a narrower check cannot cover this change'
```

It needs the toolchain pinned in `rust-toolchain.toml` (rustup installs it) and `cargo-deny`. `make doctor` lists what is missing.

## License

MIT. See [LICENSE](LICENSE).
