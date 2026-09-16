# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **The Java binding loads the library it ships.** The jar carries the native
  library under `native/<os>-<arch>/` -- the release pipeline stages every
  platform there -- but the loader only ever looked at `-Dnative.lib.dir` and
  the working directory, so a Maven Central consumer got a jar it could not
  load without pointing the JVM at a library it had to build itself. The loader
  now resolves in wickra's order: `-Dnative.lib.dir` when set, the bundled copy
  extracted to a temporary file, every `target/release` or `target/debug` up
  the tree from the working directory and the class's own location, then the
  bare name.

### Changed

- **Every README follows wickra's shape.** A cross-repo scan compared the
  heading skeleton of each README against wickra's and this repository's
  differed throughout. The root README opens as wickra's does (banner, badges,
  the one-liner, the live-demo and ecosystem lines, no separate H1), the
  License section carries wickra's wording and its `### Contribution` clause,
  and the shared sections run in wickra's order. Each binding README --
  Python, Node.js, WASM, C#, Java, Go, R, C / C++ -- is `Install`, `Quick
  start`, `Benchmark`, `Documentation`, `Security`, `Disclaimer`, `License`
  with the product's own surface and protocol notes as subsections; the
  registry pages that render them now say how to report a vulnerability and
  under which licence the package ships. `examples/README.md` lists every
  language the way wickra's does, with the commands the CI examples job runs;
  `examples/{c,csharp,go,java,r,wasm}/README.md`, `fuzz/README.md` and the
  `## Editing the docs` section of `docs/README.md` exist as they do in wickra.

### Changed

- **wickra-backtest-core 0.1.6 and wickra-exchange 0.1.5.** Both pins move from
  `=0.1.5` / `=0.1.4` to the releases the family is on; the lock follows. A
  cross-repo scan lined the 24 wickra-lib repositories up, and the only other
  thing this one spelled differently was the example job's `dotnet-version`,
  which now reads `8.0.x` like the siblings'.

### Changed

- **uv 0.12.15 for the lockfile script.** `scripts/update-lockfiles.sh`
  bootstraps 0.12.15 (was 0.12.13); the pin and all four release
  checksums move together, taken from the release's `.sha256` files.

## [0.1.0] - 2026-09-14

### Security

- **rustls 0.23.45.** RUSTSEC-2026-0285: rustls accepted TLS 1.3 handshake
  messages sent at the wrong encryption level. The lock moves to the
  patched release; nothing in the code changes.

### Fixed

- **A mutating command through the C ABI executed twice.** `wickra_copilot_command`
  ran the command on every call, and every consumer of the length-then-fill
  protocol -- Go, C#, Java, R and the C examples -- calls it twice (once for the
  length, once for the bytes), so `set_spec`, `build_context` and `reset` ran
  twice through four of the ten bindings. The handle now caches the response it
  has computed but not yet delivered and reuses it for a repeated call with the
  same command bytes, so a logical command runs exactly once however many
  buffer-sizing retries it takes (the contract gym already documents). A C ABI
  test pins it.
- **Operating-mode equivalence is tested in the core and in every binding.**
  `facts` is an alias of `build_context`, and `query` must answer the same
  against the context the handle stored as against a context passed inline;
  `operating_modes.rs` pins it in Rust over the golden corpus and each of the
  nine bindings checks it at its own boundary. The C suite
  (`examples/c/golden_test.c`, wired into ctest with a CMake-globbed spec list)
  checks golden parity and both modes without a JSON library. The golden tests
  fail on a missing corpus instead of skipping.
- **A C++ hull.** `bindings/c/include/wickra_copilot.hpp` -- header-only
  C++17, owns the handle, runs the length protocol, turns a negative return
  into an exception -- ships beside the C header and in the release archive;
  `examples/c/context.cpp` builds against it.
- **Every dependency comes from crates.io, and the sibling pins are exact.**
  `wickra-exchange` and `wickra-backtest-core` were git dependencies, which
  `cargo publish` refuses; they are the registry crates at `=0.1.4` and
  `=0.1.5`, as the released siblings pin them, and `wickra-core` and
  `wickra-data` follow their 1.0 lines. `deny.toml` no longer allows git
  sources.
- **The release front in the family shape.** The tag guard, the version gate,
  idempotent publishing of all three crates (`wickra-copilot-core`,
  `wickra-copilot-llm`, `wickra-copilot`) with CycloneDX SBOMs, the C ABI
  archives with the header and the hull, a Maven Central deploy that skips a
  version already on Central and waits as long as Central takes, the jar
  uploaded for the provenance job to attest, provenance over the nupkg, jar and
  C ABI archives, the Go mirror that builds before it pushes, and a
  `workflow_dispatch` that publishes nothing. The pom carries the release
  profile (sources, javadoc, GPG, the publishing plugin), `<scm>` and
  `<developers>` Central requires.
- **The Python 3.9 CI row runs without pytest.** pytest 9.x requires 3.10, so
  that row could only pin 8.4.2, below the fix for GHSA-6w46-j5rx-g56g with no
  backport. The 3.9 lock carries maturin only, and the row runs the same test
  modules through `bindings/python/tests/run_without_pytest.py`; 3.10 and up
  run them under pytest as before.
- **The R package builds the family way.** `configure` downloads the C ABI
  release archive (or builds it from the tag's source on r-universe's
  WebAssembly image), `configure.win` picks the architecture from
  `R.version$arch`, the exported functions carry generated `man/` pages, a
  shipped smoke test runs inside the tarball, `.Rbuildignore` is regex-safe,
  and `DESCRIPTION` states the R floor.
- CI in the family shape: the binding-surface, links and semver jobs, the
  wheel container smoke, an Examples job that holds every example to the
  version line, a WASM demo page (`examples/wasm/context.html`) whose module
  is parse-checked, CodeQL over C#, Java and C/C++ with a config that keeps
  generated code out, osv-scanner, timeouts on every job, patch-level pins,
  Dependabot over every manifest (the fuzz crate and the Go and Node examples
  included), the five repository-check scripts, `update-lockfiles.sh`, the
  detailed issue and PR templates, actionlint, CodSpeed with `criterion`
  aliased to it, zizmor's `self-repository` policy, and docs.rs metadata on
  all three crates.
- Licence texts travel with every published package (`LICENSES/`, copies in
  each crate and the Python and npm packages); the README opens with the
  quickstart and states the toolchain floors the manifests declare;
  `SECURITY.md` names the first release.
- **Two of the three published crates carried names the release could not
  upload.** `copilot-core` and `copilot-llm` are outside the org's crates.io
  token scope, which creates new crates under the `wickra-` prefix only;
  `cargo publish` on either returns 403 at upload while `--dry-run` passes,
  and because the publish jobs run in parallel the release would have landed
  on PyPI, npm, NuGet, Maven Central and the Go mirror without ever reaching
  crates.io. They are now `wickra-copilot-core` and `wickra-copilot-llm`, the
  shape of every released sibling. Directories keep their names; only the
  packages and the `wickra_copilot_core` / `wickra_copilot_llm` paths moved.
  The same audit ran across the family (xray paid for this with its first
  tag).

### Added

- The `wickra-copilot-core` deterministic core: `ContextSpec` (JSON/TOML), the six fact
  derivations (price move, order-book imbalance, liquidation cluster, funding
  flip, open-interest change, volatility spike), each with a fixed significance
  threshold and a byte-pinned English `human` sentence, assembled into a ranked
  `MarketContext`, and the `Copilot::command` JSON-over-C-ABI protocol
  (`set_spec`, `build_context`/`facts`, `query`, `reset`, `version`). The
  parallel (rayon) and sequential builds are byte-for-byte identical.
- `wickra-copilot-llm`: a separate LLM adapter — never reachable over the C ABI — with
  four provider presets (Ollama, OpenAI, Claude, Gemini) plus a custom endpoint,
  driven by one OpenAI-compatible client, configured through the
  `WICKRA_COPILOT_API_KEY` / `_BASE_URL` / `_MODEL` environment variables. Local
  by default, read-only, no SaaS.
- `wickra-copilot` CLI: `context` builds and prints the deterministic facts;
  `ask` builds the context, routes the question and asks a configured provider to
  explain it (`--spec`, `--feeds` / `--stdin`, `--format`, `--provider`).
- Ten-language surface: native Rust, Python (PyO3), Node.js (napi) and WASM
  (wasm-bindgen), plus a C ABI hub (cbindgen) backing C, C++, C#, Go, Java and R.
  Only the deterministic core is exposed.
- Question routing: `query` maps a natural-language question to the fact kinds it
  needs through a fixed keyword table, returning deterministic `ToolCall`s.
- A deterministic golden corpus (feed universe, specs, byte-exact expected
  contexts) and cross-language byte-equality tests across every binding.
- Test rigor: conformance, golden, parallel-equals-sequential, property-based
  invariants, four cargo-fuzz targets, and the `copilot-bench` criterion suite.
- One runnable "build a context" example per language, an `ask` LLM demo, and the
  core documentation set under `docs/` (architecture, facts, grounding, LLM
  adapter, tool calling, cookbook).
- CI/CD: a multi-OS test matrix across ten languages, CodeQL, OpenSSF Scorecard,
  zizmor, link-check, benchmark and metadata-audit workflows, plus an authored
  (tag-gated) release workflow.
- Repository scaffolding: Cargo workspace, supply-chain configuration
  (`deny.toml`, `osv-scanner.toml`, `lychee.toml`), lint configuration
  (`clippy.toml`), `repo-metadata.toml`, and dual `MIT OR Apache-2.0` licensing.

[Unreleased]: https://github.com/wickra-lib/wickra-copilot/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/wickra-lib/wickra-copilot/releases/tag/v0.1.0
