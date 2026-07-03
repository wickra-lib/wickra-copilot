# Threat Model

`wickra-copilot` is a local analysis tool. It folds public market data into a
structured `MarketContext` and relays it to a language model of the user's
choosing; it places no orders and opens no authenticated exchange connections.
Its attack surface has two parts: the deterministic core, dominated by the parsing
of **untrusted input** (a `ContextSpec` and feed snapshots) as it crosses the C
ABI and WASM boundary; and the LLM adapter, which handles a **secret API key** and
makes an **outbound network request** to a user-configured endpoint.

## Assets

- The **provider API key**. This *is* a secret. For the cloud providers the
  adapter holds a bearer token read from the environment; it must never leak
  (into a log, an error message, a committed file, or a request to any host other
  than the configured LLM endpoint).
- The **`ContextSpec` and feed snapshots** a caller supplies. These are inputs,
  not secrets, but a malformed or hostile one must never crash or corrupt the
  host.
- The **integrity and determinism** of the `MarketContext`: the same spec and
  feeds must always produce the same context, in every language, and identically
  between the parallel and sequential builds.
- The **host process** embedding a binding. Building a context must not be able to
  take it down (panic across FFI, unbounded allocation) or read memory it should
  not.

## Trust boundaries

- **Caller → core.** Everything arriving through `Copilot::command` (spec, feed
  snapshots, command) is untrusted and validated before use.
- **Binding → C ABI hub.** The hub is the one place `unsafe` is allowed. It wraps
  every call in `catch_unwind`, guards null pointers, and uses a length-out buffer
  protocol so no panic or invalid pointer crosses into C / Go / C# / Java / R.
- **Exchange feed → facts.** The order-book, trade, funding, open-interest and
  liquidation streams sourced through `wickra-exchange` are public market data;
  they add a network read but no credentials or orders, and their contents are
  validated like any other untrusted input.
- **Adapter → LLM endpoint.** The `copilot-llm` adapter makes an outbound HTTPS
  request carrying the rendered prompt and, for cloud providers, the API key. The
  key travels **only** to the configured `base_url`. Selecting a `custom` endpoint
  is an explicit user decision, and the key (and the market context) go to
  whatever host the user configured — so users must trust that endpoint. The
  deterministic core is entirely off this path and does no networking.

## Guarantees the code is held to

- The API key is read from the environment only, never logged, never written to a
  committed file, and transmitted only to the configured LLM `base_url`.
- `unsafe_code = "forbid"` workspace-wide; only `bindings/c` re-allows it locally.
- No panic crosses the FFI boundary; errors are returned as JSON, never as an
  abort.
- Parsing is bounded and total — a hostile spec or feed snapshot yields an error,
  not an unbounded allocation or a hang.
- The context build is deterministic: the same feeds and spec always yield the
  same `MarketContext`, and because each binding returns the core's response
  verbatim, that context is byte-identical in every language. The LLM call is
  explicitly **not** part of this guarantee.

## Out of scope

- The correctness or safety of the third-party LLM's output — it can be wrong, is
  not financial advice, and is the provider's responsibility. The copilot grounds
  the prompt in real data but does not (and cannot) guarantee the answer.
- Incorrect fact mathematics — a functional bug, handled through normal issues and
  tests, not a vulnerability.
- Vulnerabilities in third-party crates, which are tracked and triaged through
  `deny.toml` and `osv-scanner.toml`.
- Resource exhaustion a caller inflicts on **their own** process by deliberately
  feeding enormous snapshots; the core bounds its own allocations but cannot bound
  the caller's data volume.
- A user configuring a `custom` endpoint they do not trust: the tool sends the key
  where it is told, and choosing a hostile endpoint is a user-side misconfiguration,
  not a flaw in the tool.
