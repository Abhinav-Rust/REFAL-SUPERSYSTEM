# Contributing

This project is in an early rebuild phase. Contributions should prioritize correctness, tests, and clear design.

## Development

```bash
cargo fmt
cargo test
cargo run -p refal -- check examples/identity.ref
```

## Before Opening A Pull Request

- Run `cargo fmt`.
- Run `cargo test`.
- Update documentation when behavior changes.
- Add or update examples when user-facing behavior changes.
- Keep the changelog honest.

## Project Rules

- Keep implementation clean-room.
- Add tests for language behavior.
- Prefer small, reviewable changes.
- Do not add undocumented language extensions.
- Keep README status honest.
