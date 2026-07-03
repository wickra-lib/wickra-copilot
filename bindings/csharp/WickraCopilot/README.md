# Wickra Copilot — .NET (C#)

.NET bindings for the `wickra-copilot` deterministic market-context core over its
C ABI hub, via `LibraryImport` P/Invoke. Build a `Copilot` from a spec JSON,
drive it with command JSON, read back the `MarketContext` — the same protocol as
every other binding. Only the deterministic core is exposed; the LLM adapter is
never reachable over the C ABI, so the network and API key stay off this surface.

## Install

```sh
dotnet add package Wickra.Copilot
```

## Usage

```csharp
using System.Text.Json;
using Wickra.Copilot;

const string spec = "{\"symbols\":[\"BTCUSDT\"],\"lookback\":3,\"facts\":[\"price_move\"]}";
using var copilot = new Copilot(spec);

string build = JsonSerializer.Serialize(new
{
    cmd = "build_context",
    feeds = new Dictionary<string, object>
    {
        ["BTCUSDT"] = new
        {
            symbol = "BTCUSDT",
            candles = new[]
            {
                new { ts = 1, open = 100.0, high = 100.0, low = 100.0, close = 100.0, volume = 1.0 },
                new { ts = 2, open = 97.0, high = 97.0, low = 97.0, close = 97.0, volume = 1.0 },
                new { ts = 3, open = 94.0, high = 94.0, low = 94.0, close = 94.0, volume = 1.0 },
            },
        },
    },
});
Console.WriteLine(copilot.Command(build));
Console.WriteLine(Copilot.Version());
```

## API

| Member | Description |
|--------|-------------|
| `new Copilot(specJson)` | Build a copilot from a spec JSON (`ArgumentException` on an invalid spec). |
| `copilot.Command(cmdJson)` | Apply a command JSON (`set_spec`, `build_context`, `facts`, `query`, `reset`, `version`) and return the response JSON. |
| `Copilot.Version()` | The library version. |
| `copilot.Dispose()` | Free the native handle. |

Domain errors (a bad spec, an unknown command) come back in-band as
`{"ok":false,"error":...}`; only unusable arguments and caught panics are
exceptions.
