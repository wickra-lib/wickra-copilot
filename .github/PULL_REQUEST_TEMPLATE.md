<!-- Keep it short. One logical change per PR.

     Changing the release process, the bindings' shared surface, or the public
     API? There is a longer template that asks what such a change has to answer:
     reopen this pull request with ?template=detailed.md appended to the URL.
     GitHub offers no picker for a second template, so it is only reachable that
     way. -->

## What

<!-- What does this change and why? -->

## Checklist

- [ ] `cargo fmt --all` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` are clean
- [ ] `cargo test --workspace --all-features` and `--no-default-features` pass (parallel == sequential)
- [ ] `cargo deny check` is clean
- [ ] Tests added/updated (prefer hand-computed expectations for core changes)
- [ ] Facts and context stay data (a serde `ContextSpec` / `MarketContext`), never Rust closures
- [ ] The LLM adapter stays separate and is never part of the golden/determinism gate
- [ ] Binding surface mirrored across languages; golden contexts regenerated if the schema changed
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
