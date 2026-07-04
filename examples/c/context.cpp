// A minimal C++ example: build a market context through the wickra-copilot C ABI.
#include <cstddef>
#include <iostream>
#include <string>
#include <vector>

#include "wickra_copilot.h"

namespace {
const char *SPEC =
    R"({"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]})";

// A three-bar BTC dump (100 -> 94) fed inline as a build_context command.
const char *BUILD =
    R"({"cmd":"build_context","feeds":{"BTCUSDT":{"symbol":"BTCUSDT","candles":[)"
    R"({"ts":1,"open":100,"high":100,"low":100,"close":100,"volume":1},)"
    R"({"ts":2,"open":97,"high":97,"low":97,"close":97,"volume":1},)"
    R"({"ts":3,"open":94,"high":94,"low":94,"close":94,"volume":1}]}}})";

// Length-out protocol: learn the length, then read into a caller buffer.
std::string run(WickraCopilot *copilot, const char *cmd) {
    int len = wickra_copilot_command(copilot, cmd, nullptr, 0);
    if (len < 0) {
        std::cerr << "command failed: code " << len << "\n";
        return {};
    }
    std::vector<char> buf(static_cast<std::size_t>(len) + 1);
    wickra_copilot_command(copilot, cmd, buf.data(),
                           static_cast<std::size_t>(buf.size()));
    return std::string(buf.data());
}
}  // namespace

int main() {
    WickraCopilot *copilot = wickra_copilot_new(SPEC);
    if (copilot == nullptr) {
        std::cerr << "failed to build copilot\n";
        return 1;
    }

    std::string context = run(copilot, BUILD);

    std::cout << "wickra-copilot " << wickra_copilot_version() << "\n";
    std::cout << "context: " << context << "\n";

    wickra_copilot_free(copilot);
    return 0;
}
