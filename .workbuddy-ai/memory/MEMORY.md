# REFAL-SUPERSYSTEM — project memory

## Working copy (standing order, 2026-10-07)

**This directory (`C:\MY IDEAS - 2\REFAL SUPERSYSTEM -- GITHUB REPO`) is the
primary working copy** — a full clone kept in sync with `main`. Do the work here,
commit, and push to `main` (no branches, no PRs). The Chief Architect keeps it as
a complete local mirror so the project survives any loss of the remote.

## Working rhythm (standing order, 2026-10-08)

**Finish one step at a time, then report — and the report must end with the next
course of action I recommend.** The Chief Architect's convention for this repo:

1. **Do one step.** Not a plan for a step — the step, finished, committed and
   pushed.
2. **Report the step as finished**, with its gate and its measurement.
3. **Always close the report with a recommended next course of action.** Never
   end on "what would you like next?" — name the next step and say why it is the
   right one, so the Chief Architect can simply approve or redirect.
4. **Update `README.md` side by side**, as the work lands — never batched at the
   end.

## Rules of engagement

- **Ask before installing** any toolchain/dependency; state why and how it serves
  the project. Git Bash and GitHub CLI are available.
- **Commit to `main` directly**; update `README.md` and `docs/PROGRESS.md` in the
  same commit as the work.
- **Evidence or it didn't happen.** Every claim lands with a gate. A completion
  figure is raised only behind a green gate; one figure, one method.
- **Execution is autonomous; plan-level changes need approval.** Renaming the
  repo, adding a crate, altering the completion accounting → ask first.

## Current state

- **Honest completion ~90.4%** of the four-layer supersystem (one number, one
  table, in `README.md`).
- **Conformance ledger: 19 closed / 6 partial / 1 out of scope** (`E-1 … E-26`;
  `E-26` is the Principia Cybernetica network, out of scope). Source of truth:
  `docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md`.
- **E-11's negative half is built** (`597300e`): the partition carries the
  complement of its definite branches, so a callee whose last sentence is a
  catch-all is eliminated rather than left residual. **The completion figure was
  deliberately not moved** — the withheld credit in the graph-of-states row is
  §4.4's, not E-11's, and re-weighting the accounting needs the Chief Architect.
  E-11's remaining gap is the **two-level stack configuration**.
- Rust 1.99.0. Build: `cargo build -p refal`. Fast gate:
  `./target/debug/refal differential examples/differential-corpus.manifest --corpus`.
  Full suite: `cargo test --all -j 2 -- --test-threads=1` (~60 min) — run in
  background; never in parallel (OOM).
- **The `refal-cli` crate's package name is `refal`**, not `refal-cli`:
  `cargo test -p refal --test check_examples <filter>`.

## CI (GitHub Actions) — investigated 2026-10-08

`.github/workflows/ci.yml` is the only workflow: `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, then the full serial suite.

**The Chief Architect reported failure emails "for the past 2-3 days". The
investigation found only two causes, both already fixed, and 18 consecutive green
runs since 2026-10-06:**

- **2026-10-04** — `cargo fmt --check` failed: a previous session committed
  unformatted code. Fixed by formatting. **Always run `cargo fmt --check` before
  pushing.**
- **2026-10-05 (twice)** — `strict_mode_has_no_false_positives_on_the_corpus`
  rejected `examples/specialise-template.ref`, whose `MARKER` placeholder makes
  `<Run MARKER e.In>` provably always-fail. The lint is *correct*; the file is a
  template, not a program, and is allow-listed in that test's `known_defective`.

**Why the impression persisted:** GitHub emails on *failure only*. There is no
"CI passed" mail, so a green streak is invisible — check `gh run list` rather
than trusting the inbox.

**Latent flake, fixed:** the job had `timeout-minutes: 45` while runs measured
**34-43 minutes** — a two-minute margin, shrinking with every test added (the
suite grew 383 → 389 tests on 2026-10-08). Raised to **90**.

## Assets added

- `crates/xtask` — the repository's tooling, in Rust. Six tasks via
  `cargo xtask <task>` (alias in `.cargo/config.toml`): `gen-readme-diagrams`
  (the theme-aware SVG family, `docs/images/*-{light,dark}.svg`, from one
  description — edit the task, not the SVGs; verify by regenerating and
  `git diff docs/images`, which must be empty), `sweep`, `profile`, `perf`,
  `package`, `fetch-sources`. **The repo is 100% Rust** as of 2026-10-09 — no
  Python, no Shell. Verify diagrams with headless Chrome → PNG → read.

## Reference material

- `C:\MY IDEAS - 2\VT- CS+PW` — 80 primary Turchin works across four domains,
  plus `README-FOR-REFAL-SUPERSYSTEM.md` (a reading guide mapping each `E-` row to
  its source). A derived archival edition, not scanned primaries: re-check exact
  wording against `docs/turchin/pdf/`.
