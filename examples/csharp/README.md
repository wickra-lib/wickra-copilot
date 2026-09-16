# Wickra Copilot examples — C#

Runnable C# examples for the [Wickra Copilot C# binding](../../bindings/csharp). The binding consumes the C ABI
library through P/Invoke, so build it once before running anything:

```bash
cargo build -p wickra-copilot-c --release
```

## Run

As the CI examples job runs it, from the repository root:

```bash
dotnet run --project examples/csharp/Context
```

## The examples

| Example | What it does |
|---------|--------------|
| `Context/Program.cs` | A runnable .NET example: build a market context through the binding. |
