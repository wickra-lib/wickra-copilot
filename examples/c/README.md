# Wickra Copilot — C / C++ examples

The Wickra Copilot C ABI is a single shared/static library plus a generated header
([`bindings/c/include/wickra_copilot.h`](../../bindings/c/include/wickra_copilot.h)). Any C-capable
language links against the same artifact; these examples show the plain-C path
and, through [`wickra_copilot.hpp`](../../bindings/c/include/wickra_copilot.hpp), the C++ one.

## Build the library

From the workspace root:

```sh
cargo build -p wickra-copilot-c --release
```

This produces, in `target/release/`:

| Platform | Shared library | Link target |
|----------|----------------|-------------|
| Linux    | `libwickra_copilot.so`     | `-lwickra_copilot` |
| macOS    | `libwickra_copilot.dylib`  | `-lwickra_copilot` |
| Windows (MSVC) | `wickra_copilot.dll` | `wickra_copilot.dll.lib` (import lib) |

A static library (`libwickra_copilot.a` / `wickra_copilot.lib`) is emitted alongside.

## Build and run the examples

### With CMake (portable, used by CI)

```sh
cmake -S examples/c -B examples/c/build
cmake --build examples/c/build --config Release
ctest --test-dir examples/c/build -C Release --output-on-failure
```

### Directly with a compiler

```sh
# Linux / macOS
cc examples/c/context.c -I bindings/c/include -L target/release -lwickra_copilot -lm -o context
LD_LIBRARY_PATH=target/release ./context        # macOS: DYLD_LIBRARY_PATH

# Windows (MinGW gcc, linking the DLL directly)
gcc examples/c/context.c -I bindings/c/include target/release/wickra_copilot.dll -lm -o context.exe
```

## The examples

| Example | What it does |
|---------|--------------|
| `context.c` | A minimal C example: build a market context through the wickra-copilot C ABI. |
| `context.cpp` | A minimal C++ example: build a market context, then ask the same question against the stored context and against the context passed inline -- both through the C++ hull. |

## Usage shape

Every call follows the same handle discipline: construct from a spec JSON, drive
with command JSON, read the response, free the handle exactly once. `wickra_copilot.h` is
the whole contract; the C++ header, where one ships, wraps the handle in a
move-only RAII type. See [`bindings/c/README.md`](../../bindings/c/README.md).
