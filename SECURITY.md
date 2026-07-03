# Security Policy

`wickra-copilot` is a local analysis tool: it builds a deterministic market
context from public microstructure feeds (via `wickra-exchange`) and relays it to
a language model of the user's choosing. It places no orders and opens no
authenticated exchange connections — its feeds read only public market data. Two
things widen the attack surface beyond the pure data core: the parsing of
untrusted `ContextSpec` and feed data as it crosses the C ABI and WASM boundary,
and the LLM adapter, which reads an API key from the environment and makes an
outbound HTTPS request to a user-configured endpoint. See
[THREAT_MODEL.md](THREAT_MODEL.md) for the asset inventory and trust boundaries.

## Supported versions

This project is pre-release. Security fixes target the `main` branch and the most
recent published version once a release exists.

| Version | Supported |
|---------|-----------|
| `main`  | ✅        |
| `0.1.x` (upcoming) | ✅ |

## API key handling

The LLM adapter (`copilot-llm`) needs a provider API key for the cloud providers
(OpenAI / Claude / Gemini; Ollama is local and needs none). That key is treated
as a secret end to end:

- **Environment only.** The key is read from an environment variable
  (`WICKRA_COPILOT_API_KEY`); it is never a command-line argument, never written
  to a config file the tool commits, and `.env` is git-ignored.
- **Never logged.** The key is not printed, echoed, included in error messages,
  or written to any log or trace, on any code path.
- **Never transmitted anywhere except the configured endpoint.** The only network
  destination is the LLM `base_url` you select (a provider preset or your own
  `custom` URL). The deterministic context core does no networking at all.
- **Read-only market data.** No exchange credentials are used or required; the
  copilot never authenticates to a venue and never trades.

## Reporting a vulnerability

**Please do not open a public issue, pull request or discussion for security
problems.** Report privately through either channel:

- GitHub → the repository's **Security** tab → **Report a vulnerability**
  (private advisory), or
- email **support@wickra.org**.

Include a description, affected version/commit, reproduction steps and impact.

We aim to acknowledge within a few days, agree a disclosure timeline, and credit
reporters who wish to be named once a fix ships.

## Scope

In scope: memory-safety or panic-across-FFI flaws in the C ABI hub and its buffer
protocol, denial-of-service through a hostile `ContextSpec` or feed snapshot (for
example unbounded allocation while parsing), any input that makes a binding return
a corrupted or non-deterministic `MarketContext`, and any leak of the API key
(logging, transmission to an unintended host, or exposure through an error
message). Out of scope: incorrect indicator mathematics (a functional bug, not a
vulnerability), the correctness or safety of the third-party LLM's output (it is
not financial advice and is the provider's responsibility), and advisories in
third-party crates that are already tracked and triaged.

## Vulnerability disclosure (VEX)

This repository ships a machine-readable VEX record in
[`osv-scanner.toml`](osv-scanner.toml), kept in lock-step with the cargo-deny
advisory ignore list in [`deny.toml`](deny.toml). Any advisory assessed as not
affecting `wickra-copilot` is documented there with a reason, so downstream
scanners see an explicit, auditable justification rather than an unexplained
suppression.
