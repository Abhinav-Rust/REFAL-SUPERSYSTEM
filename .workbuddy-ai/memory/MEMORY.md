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

- **Honest completion ~91.8%** of the four-layer supersystem (one number, one
  table, in `README.md`). The table's twelve credits sum to **91.80** — **the
  published figure must always equal the table's sum** (a 0.27 discrepancy was
  found and fixed on 2026-10-09; the runtime row earned 13.51 → 13.65 and the
  conformance/release row 2.66 → 2.80 on 2026-10-10). The **`status` SVG pair is
  hand-committed**, and when the figure moves **only its `<desc>` and headline
  change — never the bars** (that is what `0e21285`, `015a263` and `ccc4193` all
  did; a bar edit this session was reverted).
- **A committed Refal fixture *can* do file I/O** (2026-10-10): the conformance
  harness `run_with_closed_stdin` runs every `run` row in a fresh temp directory
  (set as `current_dir`, removed after), so a fixture may use a **relative** file
  name. `examples/builtin-file-io-conformance.ref` is the pattern; the `unit` row
  stays for assertions a program cannot print (exact file bytes).
- **A block sentence carrying a condition now runs on the work list** (2026-10-10,
  runtime row closed). `ConditionEval` carries a `ConditionOwner` (`Function` or
  `Block`) so a failing chain continues into the right next sentence;
  `terms_are_worklist_safe` no longer rejects a block with conditions, which is
  what had pushed every function containing one off the work list. Gate:
  `a_block_sentence_that_carries_a_condition_runs_on_the_work_list` (100,000
  steps; overflows the stack against the old code).
- **The README is visual-first** (2026-10-10, `1429e8f`): 13 generated panels
  (`cargo xtask gen-readme-diagrams`), narrative in diagrams with one-line
  captions, detailed prose collapsed into `<details>`. `docs/SEO.md` holds the
  keyword strategy. **Canva is deliberately not used** — code-generated
  theme-aware SVG is the repo's convention and is strictly better here.
- **`refal feasibility <file.ref> [--certificate]`** (2026-10-09) — Tier 1
  feasibility and termination with certificates. Per sentence: `feasible` (ground
  witness, re-checked by `verify`) / `infeasible` (shadowing proof) / `unproven`
  (named). Per function: `Terminating { components }` / `NonRecursive` /
  `Unproven`. Termination is **size change over the strongly connected
  components** (`CallClass` strict/nonstrict/unknown) under a **ranking** —
  measure 0 is the whole argument, measure k>0 its k-th top-level term, and a
  ranking may be a **lexicographic pair** `k+l` — so **mutual recursion is
  covered** and a call that rebuilds its argument can still be proved. Module:
  `crates/refal-semantics/src/feasibility.rs`. Measured: 0 infeasible across the
  77 non-`bad-*` examples; `compiler.ref` 330 unproven functions (343 before
  pairs, 439 before per-position measures, 129 for the earlier *unsound*
  version).
- **THE MEASURE DIRECTION IS CLOSED (2026-10-09).** A lexicographic pair is
  `unknown` wherever its *first* measure is, so pairs can break non-decreasing
  **cycles** but can **never bound a call that no measure bounds**. Pinned as
  `a_pair_can_never_bound_a_call_its_first_measure_cannot`. Do NOT try deeper
  paths, more measures, or other combinations: the compiler's 21 unbounded calls
  are a call in argument position (`<DsScan <DsBump …>>`), which has no static
  size. The driven-graph norm was also measured and refuted. **The measure must
  bound the call itself** — or the technique must change.
- **Soundness over coverage, always.** An earlier version of that analysis
  collected only self-calls and so reported mutual loops as terminating; it was
  replaced, and the unproven count rose 129 → 439. Record the worse number.
- **The repository is 100% Rust** (2026-10-09) — no Python, no Shell; tooling is
  `crates/xtask` via `cargo xtask <task>`. **The GitHub language bar is pinned to
  match** (2026-10-10, `7319800`): `.gitattributes` marks `.ref`/`.md`/`.manifest`
  as `linguist-documentation` and `docs/images/*.svg` as
  `linguist-generated`, so the bar is a property of the file, not of Linguist's
  heuristics. The `Rust 97.1% / Python 2.4% / Shell 0.5%` the Chief Architect saw
  was stale cache from before `8151ce1`; Linguist recomputes on push.
- **README visuals are code-generated** (2026-10-10): `cargo xtask
  gen-readme-diagrams` emits ten panels (was nine) as light/dark SVG pairs from
  one description; the tenth, `glance`, is the five-tile at-a-glance dashboard
  under the hero. **Rule: edit `crates/xtask/src/diagrams.rs`, never the SVGs;
  regeneration must leave `git diff docs/images` empty.** Seven older panels
  (`hero`, `status`, `layers`, `pipeline`, `prover`, `fixpoint`, `metasystem`) are
  still hand-committed SVG, not generated.
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
