# A runnable R example: build a market context through the binding.
#
#   cargo build -p wickra-copilot-c --release
#   export WKCOPILOT_LIB="$PWD/target/release"
#   export LD_LIBRARY_PATH="$WKCOPILOT_LIB:$LD_LIBRARY_PATH"   # PATH on Windows
#   R CMD INSTALL bindings/r
#   Rscript examples/r/context.R

library(wickracopilot)

spec <- '{"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]}'

build_cmd <- paste0(
  '{"cmd":"build_context","feeds":{"BTCUSDT":{"symbol":"BTCUSDT","candles":[',
  '{"ts":1,"open":100,"high":100,"low":100,"close":100,"volume":1},',
  '{"ts":2,"open":97,"high":97,"low":97,"close":97,"volume":1},',
  '{"ts":3,"open":94,"high":94,"low":94,"close":94,"volume":1}]}}}'
)

copilot <- wkcopilot_new(spec)
response <- wkcopilot_command(copilot, build_cmd)

cat("wickra-copilot", wkcopilot_version(), "\n")
cat(response, "\n")
