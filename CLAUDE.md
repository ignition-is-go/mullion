# mullion

A Leptos CSR library for splittable panes with activity bars. Consumed by
`rship-leptos-ui` and `levi` via path or git dependency — **never** from
crates.io, where the `mullion` name is squatted.

## Gotchas

- **Conventional commits are required** — `cargo flux version` derives the next
  semver from commit messages (`feat:` minor, `fix:` patch, `feat!:` /
  `BREAKING CHANGE:` major). `chore(release):` commits are stamps flux creates
  itself; never write one by hand.
- **wasm is the real target.** Native `cargo check` passes but doesn't exercise
  the `web-sys` / `wasm-bindgen` paths. Gate on `cargo flux run check-wasm`.
- **Adding a field to a public struct breaks literal consumers.** `MullionTheme`,
  `ActivityDef`, and friends are built with struct literals downstream (e.g.
  `rship-leptos-ui`'s `theme_bridge.rs`), so a new field is a breaking change
  even though it looks additive. Prefer a builder, a component prop, or a CSS
  custom property with a `var(--x, fallback)` default.
- **The stacking contract is load-bearing.** Pane content is `isolation:isolate`,
  so an activity's z-index can never compete with chrome (handles 5, activity
  bar 10, drop overlay 20). Content that must escape the pane goes through
  `MullionOverlay`, not a bigger number. `DropOverlay` is deliberately a sibling
  *outside* the isolate so drag chrome stays above content — don't fold it in.
  See the README's stacking section before touching any z-index.
- **`examples/demo` is its own workspace**, so it is not covered by flux tasks
  run from the repo root. Build it separately with `trunk build` inside it.

## Build

- `cargo flux run check-wasm` — the real gate.
- `cargo flux run check` / `cargo flux run test` / `cargo flux run lint`.
- Demo: `trunk serve` inside `examples/demo/`.

<!-- levi:begin -->
## Task tracking (levi)

This repo tracks tasks with levi, a git-aware issue tracker. State lives in
the repo itself (`refs/levi/events`); status is resolved against git
ancestry, so a task closed at commit X counts as closed only on checkouts
that contain X. Every read command takes `--json` (stable schemas) — prefer
it when parsing.

- **Pick work**: `levi next --claim --json` returns the most important
eligible task, claims it for this dev/machine/worktree (so parallel agents
never grab the same task), and tells you why it ranked first. If you stop
working on a task, release it: `levi drop <id>`.
- **Inspect**: `levi ls --json` (open on this checkout), `levi show <id>
--json` (body, deps, claim, comments, status history).
- **Create**: `levi add "title" [-p p0..p3] [-b body] [-l label]
[--dep <blocker-id>]` — file follow-ups you discover instead of fixing
drive-by; link blockers with `--dep`/`levi dep add`.
- **Complete**: commit the work first, then `levi close <id>` — the close
anchors at HEAD, so it only applies where the fixing commit exists
(feature-branch closes stay open on main until merged; that is correct).
`--no-anchor` is only for tasks unrelated to code state.
- **Reopen** regressions with `levi reopen <id>`; leave context with
`levi comment <id> "text"`.
- Sync is opportunistic after every mutation; `levi sync` forces a full
git-remote + hub exchange.
- **Cross-project**: file upstream bugs with `levi add --project <name>
"title"`; link with `levi dep add <id> --on <project>/lv-xxxx --via
"<how this repo consumes that project>"`. When a foreign blocker
closes, verify the fix is actually reachable through the `via`
mechanism (published release, updated pin, ...) before starting work.
<!-- levi:end -->
