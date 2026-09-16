# Wickra Copilot examples — Go

Runnable Go examples for the [Wickra Copilot Go binding](../../bindings/go). The binding links against the
prebuilt C ABI library, so build and stage it once before running anything:

```bash
cargo build -p wickra-copilot-c --release
mkdir -p bindings/go/lib/linux_amd64
cp target/release/libwickra_copilot.so bindings/go/lib/linux_amd64/
```

## Run

As the CI examples job runs it, from the repository root:

```bash
cd examples/go && go run .
```

## The examples

| Example | What it does |
|---------|--------------|
| `context.go` | A runnable Go example: build a market context through the binding. |
