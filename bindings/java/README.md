# Wickra Copilot — Java

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

## Usage

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

## API

| Member | Description |
|--------|-------------|
| `new Copilot(specJson)` | Build a copilot from a spec JSON (`IllegalArgumentException` on an invalid spec). |
| `copilot.command(cmdJson)` | Apply a command JSON (`set_spec`, `build_context`, `facts`, `query`, `reset`, `version`) and return the response JSON. |
| `Copilot.version()` | The library version. |
| `copilot.close()` | Free the native handle (`AutoCloseable`). |

Domain errors (a bad spec, an unknown command) come back in-band as
`{"ok":false,"error":...}`; only unusable arguments and caught panics are
exceptions.

## Test

```sh
cargo build -p wickra-copilot-c --release
mvn -q test
```
