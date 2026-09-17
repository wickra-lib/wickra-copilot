# Wickra Copilot examples — R

Runnable R examples for the [Wickra Copilot R binding](../../bindings/r). The package compiles a thin
`.Call` glue layer against the C ABI library, so build the library and install
the package first (the CI examples job does exactly this):

```bash
cargo build -p wickra-copilot-c --release
R CMD INSTALL bindings/r
```

## Run

```bash
Rscript examples/r/context.R
```

## The examples

| Example | What it does |
|---------|--------------|
| `context.R` | A runnable R example: build a market context through the binding. |
