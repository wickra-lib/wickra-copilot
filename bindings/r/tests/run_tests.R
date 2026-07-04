## Plain-R tests for the wickra-copilot R binding (no testthat dependency).
## Mirrors the Rust/Python/Node/Go/C#/Java tests and doubles as the completeness
## guard: it exercises the full public surface (version + new + command).

library(wickracopilot)

spec <- '{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]}'

candle <- function(ts, close) {
  paste0(
    '{"ts":', ts, ',"open":', close, ',"high":', close, ',"low":', close,
    ',"close":', close, ',"volume":1.0}'
  )
}

## BTC drops 6% over three bars -> one significant price-move fact.
build_context <- paste0(
  '{"cmd":"build_context","feeds":{"BTCUSDT":{"symbol":"BTCUSDT","candles":[',
  candle(1, "100.0"), ',', candle(2, "97.0"), ',', candle(3, "94.0"), ']}}}'
)

## version
stopifnot(nzchar(wkcopilot_version()))

## build_context roundtrip
copilot <- wkcopilot_new(spec)
raw <- wkcopilot_command(copilot, build_context)
stopifnot(grepl('"price_move"', raw, fixed = TRUE))
stopifnot(grepl('"symbol":"BTCUSDT"', raw, fixed = TRUE))

## query routes to the price-move tool call
query <- wkcopilot_command(copilot, '{"cmd":"query","question":"why did BTC dump"}')
stopifnot(grepl('"tool_calls"', query, fixed = TRUE))
stopifnot(grepl('"price_move"', query, fixed = TRUE))

## invalid spec raises
stopifnot(inherits(try(wkcopilot_new("not json"), silent = TRUE), "try-error"))

## an unknown command is an in-band error, not a hard error
inband <- wkcopilot_command(copilot, '{"cmd":"nope"}')
stopifnot(grepl('"ok":false', inband, fixed = TRUE))

## cross-language golden parity: build the copilot from each committed
## golden/specs/*.json, run build_context over the shared golden/feeds.json and
## read back the context, and assert the response equals
## golden/expected/<spec>.json byte-for-byte. The binding returns the core's
## compact command output verbatim, so byte equality is the exact cross-language
## parity check. The fixtures arrive in a later phase; until then the golden
## section is skipped.
golden_dir <- function() {
  d <- normalizePath(getwd(), mustWork = FALSE)
  for (i in seq_len(8)) {
    g <- file.path(d, "golden")
    if (dir.exists(file.path(g, "specs"))) {
      return(g)
    }
    d <- dirname(d)
  }
  NULL
}

g <- golden_dir()
if (!is.null(g)) {
  feeds <- trimws(paste(
    readLines(file.path(g, "feeds.json"), warn = FALSE), collapse = "\n"
  ))
  build_all <- paste0('{"cmd":"build_context","feeds":', feeds, '}')
  for (spec_path in list.files(file.path(g, "specs"), pattern = "\\.json$", full.names = TRUE)) {
    name <- basename(spec_path)
    spec_json <- paste(readLines(spec_path, warn = FALSE), collapse = "\n")
    expected <- trimws(paste(
      readLines(file.path(g, "expected", name), warn = FALSE), collapse = "\n"
    ))
    gcopilot <- wkcopilot_new(spec_json)
    got <- wkcopilot_command(gcopilot, build_all)
    stopifnot(identical(trimws(got), expected))
  }
}

cat("wickra-copilot R tests passed\n")
