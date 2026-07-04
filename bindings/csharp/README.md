# Wickra Copilot — C#

.NET bindings for the `wickra-copilot` deterministic market-context core over its
C ABI hub, via `LibraryImport` P/Invoke. Build a `Copilot` from a spec JSON, drive
it with command JSON, read back the `MarketContext` — the same protocol as every
other binding. Only the deterministic core is exposed; the LLM adapter is never
reachable over the C ABI, so the network and API key stay off this surface.

## Requirements

- .NET 8 SDK.
- The native C ABI library (`wickra_copilot`) built by
  `cargo build -p wickra-copilot-c --release`. The resolver looks next to the
  assembly and in the workspace `target/release`.

## Usage

```csharp
using Wickra.Copilot;

const string spec = "{\"symbols\":[\"BTCUSDT\"],\"lookback\":3,\"facts\":[\"price_move\"]}";
using var copilot = new Copilot(spec);

const string feeds = "{\"cmd\":\"build_context\",\"feeds\":{\"BTCUSDT\":{\"symbol\":\"BTCUSDT\",\"candles\":["
    + "{\"ts\":1,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1},"
    + "{\"ts\":2,\"open\":97,\"high\":97,\"low\":97,\"close\":97,\"volume\":1},"
    + "{\"ts\":3,\"open\":94,\"high\":94,\"low\":94,\"close\":94,\"volume\":1}]}}}";

Console.WriteLine(copilot.Command(feeds));
Console.WriteLine(Copilot.Version());
```

## API

| Member | Description |
|--------|-------------|
| `new Copilot(specJson)` | Build a copilot from a spec JSON (`ArgumentException` on an invalid spec). |
| `copilot.Command(cmdJson)` | Apply a command JSON (`set_spec`, `build_context`, `facts`, `query`, `reset`, `version`) and return the response JSON. |
| `Copilot.Version()` | The library version. |
| `copilot.Dispose()` | Free the native handle (`IDisposable`). |

Domain errors (a bad spec, an unknown command) come back in-band as
`{"ok":false,"error":...}`; only unusable arguments and caught panics are
exceptions.

## Test

```sh
cargo build -p wickra-copilot-c --release
dotnet test
```
