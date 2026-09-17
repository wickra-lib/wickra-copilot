<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Copilot — a local market copilot grounded in real order book, liquidation and funding microstructure" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/ci.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-copilot)
[![GitHub release](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/release.svg)](https://github.com/wickra-lib/wickra-copilot/releases/latest)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/license.svg)](https://github.com/wickra-lib/wickra-copilot#license)

# Wickra Copilot — C / C++

---

> **▶ Live demo:** all 514 indicators over real Binance market data, computed live in your browser — **[live.wickra.org](https://live.wickra.org)** · zero backend, powered by `wickra-wasm`.

**A local market copilot: an LLM grounded in real order book, liquidation and funding microstructure — for C / C++. `cargo build -p wickra-copilot-c --release` — a prebuilt shared/static library plus a generated `wickra_copilot.h`, no system dependencies.**

The C ABI is the hub every C-capable language (C, C++, C#, Go, Java, R) links
against. It exposes `wickra-copilot-core` as a tiny, JSON-shaped surface built as both a
`cdylib` (dynamic library) and a `staticlib`. Only the deterministic core is
exposed — the LLM adapter (`wickra-copilot-llm`) is never reachable over the FFI, so the
network and API key stay off this surface entirely.

## Install

Grab the prebuilt header + library for your platform from the
[GitHub releases](https://github.com/wickra-lib/wickra-copilot/releases) — each archive
has `wickra_copilot.h`, the C++ wrapper where the binding ships one, and the shared/static
library — or build from source:

```bash
cargo build -p wickra-copilot-c --release
# -> target/release/libwickra_copilot.{so,dylib} or wickra_copilot.dll (+ import lib) + a staticlib
```

Then compile against the header and link the library.

## Quick start

[`examples/c/context.c`](https://github.com/wickra-lib/wickra-copilot/blob/main/examples/c/context.c) is the runnable example the CI smoke job executes; in full:

```c
/* A minimal C example: build a market context through the wickra-copilot C ABI. */
#include <stdio.h>
#include <stdlib.h>

#include "wickra_copilot.h"

static const char *SPEC =
    "{\"symbols\":[\"BTCUSDT\"],\"lookback\":3,\"facts\":[\"price_move\"]}";

/* A three-bar BTC dump (100 -> 94) fed inline as a build_context command. */
static const char *BUILD =
    "{\"cmd\":\"build_context\",\"feeds\":{\"BTCUSDT\":{\"symbol\":\"BTCUSDT\","
    "\"candles\":["
    "{\"ts\":1,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1},"
    "{\"ts\":2,\"open\":97,\"high\":97,\"low\":97,\"close\":97,\"volume\":1},"
    "{\"ts\":3,\"open\":94,\"high\":94,\"low\":94,\"close\":94,\"volume\":1}]}}}";

/* Length-out protocol: learn the length, then read into a caller buffer.
   Returns a malloc'd NUL-terminated string the caller must free, or NULL. */
static char *run(WickraCopilot *copilot, const char *cmd) {
    int len = wickra_copilot_command(copilot, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    wickra_copilot_command(copilot, cmd, buf, (size_t)len + 1);
    return buf;
}

int main(void) {
    WickraCopilot *copilot = wickra_copilot_new(SPEC);
    if (!copilot) {
        fprintf(stderr, "failed to build copilot\n");
        return 1;
    }

    char *context = run(copilot, BUILD);
    if (!context) {
        wickra_copilot_free(copilot);
        return 1;
    }

    printf("wickra-copilot %s\n", wickra_copilot_version());
    printf("context: %s\n", context);

    free(context);
    wickra_copilot_free(copilot);
    return 0;
}
```

### Surface

```c
#include "wickra_copilot.h"

WickraCopilot *wickra_copilot_new(const char *spec_json);
void           wickra_copilot_free(WickraCopilot *handle);
int32_t        wickra_copilot_command(WickraCopilot *handle,
                                     const char *cmd_json,
                                     char *out, size_t cap);
const char    *wickra_copilot_version(void);
```

- **`wickra_copilot_new`** builds a copilot from a spec JSON (`""` or `"{}"` for
  an empty handle whose spec is set later). Returns `NULL` if the argument is
  null, not UTF-8, or not a valid spec.
- **`wickra_copilot_free`** destroys a handle (null is a no-op).
- **`wickra_copilot_command`** applies a command JSON and writes the response
  JSON into the caller's buffer using a length-out protocol (below).
- **`wickra_copilot_version`** returns a static, NUL-terminated version string
  (do not free).

### Command / response protocol

Everything after construction goes through `wickra_copilot_command`. Commands are
JSON objects with a `"cmd"` field: `set_spec`, `build_context`, `facts`, `query`,
`reset`, `version`. Responses are JSON, e.g. a `MarketContext` for
`build_context`/`facts`, `{"tool_calls":[...]}` for `query`, `{"version":...}`
for `version`, or `{"ok":true}` for a mutation. A bad spec or unknown command
comes back in-band as `{"ok":false,"error":...}`.

The response is returned via a caller-owned buffer with a length-out protocol —
the callee never allocates memory the caller must free:

1. Call with `out = NULL`, `cap = 0` to learn the response length `len`
   (excluding the terminating NUL).
2. Allocate `len + 1` bytes and call again; the response plus a NUL is written.

Whenever `len < cap`, the response is written on that call, so a
sufficiently-large buffer needs only one call.

A mutating command (`set_spec`, `build_context`, `reset`) is executed exactly
once across those calls: the handle caches the response it has computed but not
yet delivered, and a repeated call with the same command bytes reuses it
instead of re-executing. Once the response has been written to a buffer the
cache is cleared, so the next identical command executes freshly.

Return codes:

| Return   | Meaning                                             |
|----------|-----------------------------------------------------|
| `>= 0`   | Response length in bytes (excluding the NUL).       |
| `-1`     | A required pointer (`handle` or `cmd_json`) is null. |
| `-2`     | `cmd_json` is not valid UTF-8.                       |
| `-3`     | A panic was caught at the boundary.                 |

Domain errors (a bad spec, an unknown command) are **not** negative — they come
back in-band as `{"ok":false,"error":...}` JSON in the buffer.

### C++

`include/wickra_copilot.hpp` is a header-only C++17 hull over the same four
functions: `wickra::Copilot` owns and frees the handle, `command` runs the
length-out protocol for you, and a negative return becomes a
`wickra::CopilotError`. In-band refusals (`{"ok":false,...}`) are returned as
strings, not thrown. `examples/c/context.cpp` builds against it.

### Header generation

`include/wickra_copilot.h` is generated with [cbindgen] and committed; CI fails
if it drifts from the source. Regenerate after changing the ABI:

```sh
cbindgen --config cbindgen.toml --crate wickra-copilot-c --output include/wickra_copilot.h
```

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the C ABI itself, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-copilot/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-copilot>
- **Docs** (guides, spec reference, cookbook): <https://copilot.wickra.org>
- **Runnable example:** [`examples/c/`](https://github.com/wickra-lib/wickra-copilot/tree/main/examples/c)

Wickra Copilot ships native bindings for Python, Node.js, WASM and Rust, plus a C ABI hub that any
C-capable language (C, C++, C#, Go, Java, R) links against — all forwarding to the
same data-driven, `unsafe`-forbidden Rust core.

## Security

Found a security issue? **Please don't open a public issue.** Report it privately
via the repository's *Security* tab (*"Report a vulnerability"*) or email
**support@wickra.org** with a subject line starting `[wickra security]`. Full
policy: <https://github.com/wickra-lib/wickra-copilot/blob/main/SECURITY.md>.

## Disclaimer

Wickra Copilot is analysis software: it builds a deterministic market context and
relays it to a language model of your choosing. It is provided "as is", without
warranty of any kind. LLM output can be wrong and is **not financial advice**; the
copilot only reports facts and places no orders. Trading carries risk of loss;
review the code and use at your own discretion.

## License

Licensed under either of [Apache-2.0](https://github.com/wickra-lib/wickra-copilot/blob/main/LICENSE-APACHE)
or [MIT](https://github.com/wickra-lib/wickra-copilot/blob/main/LICENSE-MIT) at your option.

[cbindgen]: https://github.com/mozilla/cbindgen
