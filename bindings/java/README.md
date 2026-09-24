<p align="center">
  <a href="https://wickra.org"><img src="https://raw.githubusercontent.com/wickra-lib/.github/main/profile/wickra-banner.webp?v=514-7" alt="Wickra Copilot — a local market copilot grounded in real order book, liquidation and funding microstructure" width="100%"></a>
</p>

[![CI](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/ci.svg)](https://github.com/wickra-lib/wickra-copilot/actions/workflows/ci.yml)
[![codecov](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/codecov.svg)](https://codecov.io/gh/wickra-lib/wickra-copilot)
[![Maven Central](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/maven.svg)](https://central.sonatype.com/artifact/org.wickra/wickra-copilot)
[![License: MIT OR Apache-2.0](https://raw.githubusercontent.com/wickra-lib/.github/main/profile/badges/wickra-copilot/license.svg)](https://github.com/wickra-lib/wickra-copilot#license)

# Wickra Copilot — Java

---

> **▶ Live demo:** all 514 indicators over real Binance market data, computed live in your browser — **[live.wickra.org](https://live.wickra.org)** · zero backend, powered by `wickra-wasm`.

**A local market copilot: an LLM grounded in real order book, liquidation and funding microstructure — for Java. `org.wickra:wickra-copilot` — prebuilt native library inside the jar, no JNI, no system dependencies.**

JVM bindings for the `wickra-copilot` deterministic market-context core over its
C ABI hub, via the Foreign Function & Memory API (Panama / FFM). Build a
`Copilot` from a spec JSON, drive it with command JSON, read back the
`MarketContext` — the same protocol as every other binding. Only the
deterministic core is exposed; the LLM adapter is never reachable over the C ABI,
so the network and API key stay off this surface.

## Requirements

- JDK 22+ (FFM is stable since JDK 22).
- The native C ABI library (`wickra_copilot`) built by
  `cargo build -p wickra-copilot-c --release`. Point the JVM at it with
  `-Dnative.lib.dir=<dir>` (defaults to the workspace `target/release`).

## Install

Maven:

```xml
<dependency>
  <groupId>org.wickra</groupId>
  <artifactId>wickra-copilot</artifactId>
  <version>0.1.3</version>
</dependency>
```

Gradle:

```kotlin
implementation("org.wickra:wickra-copilot:0.1.3")
```

The native library ships prebuilt per platform inside the jar and is
extracted automatically on first use. There is nothing to compile.

### Building from this repository (contributors)

```sh
cargo build -p wickra-copilot-c --release
mvn -q test
```

## Quick start

```java
import org.wickra.copilot.Copilot;

String spec = "{\"symbols\":[\"BTCUSDT\"],\"lookback\":3,\"facts\":[\"price_move\"]}";
try (Copilot copilot = new Copilot(spec)) {
    String feeds = "{\"cmd\":\"build_context\",\"feeds\":{\"BTCUSDT\":{\"symbol\":\"BTCUSDT\",\"candles\":["
            + "{\"ts\":1,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1},"
            + "{\"ts\":2,\"open\":97,\"high\":97,\"low\":97,\"close\":97,\"volume\":1},"
            + "{\"ts\":3,\"open\":94,\"high\":94,\"low\":94,\"close\":94,\"volume\":1}]}}}";
    System.out.println(copilot.command(feeds));
    System.out.println(Copilot.version());
}
```

Run with native access enabled:

```sh
java --enable-native-access=ALL-UNNAMED -Dnative.lib.dir=../../target/release ...
```

### API

| Member | Description |
|--------|-------------|
| `new Copilot(specJson)` | Build a copilot from a spec JSON (`IllegalArgumentException` on an invalid spec). |
| `copilot.command(cmdJson)` | Apply a command JSON (`set_spec`, `build_context`, `facts`, `query`, `reset`, `version`) and return the response JSON. |
| `Copilot.version()` | The library version. |
| `copilot.close()` | Free the native handle (`AutoCloseable`). |

Domain errors (a bad spec, an unknown command) come back in-band as
`{"ok":false,"error":...}`; only unusable arguments and caught panics are
exceptions.

## Benchmark

Every binding forwards to the same data-driven Rust core, so what this one adds is
the call overhead of the Java Foreign Function & Memory API over the C ABI, not a different result. The core's throughput is
measured by the repository's benchmark suite and the nightly `bench.yml` run; the
numbers, the machine and how to reproduce them are in the repository
[BENCHMARKS.md](https://github.com/wickra-lib/wickra-copilot/blob/main/BENCHMARKS.md).

## Documentation

The full guide, the spec reference and the API documentation live in the main
repository and the documentation site:

- **Repository:** <https://github.com/wickra-lib/wickra-copilot>
- **Docs** (guides, spec reference, cookbook): <https://copilot.wickra.org>
- **Runnable example:** [`examples/java/`](https://github.com/wickra-lib/wickra-copilot/tree/main/examples/java)

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
