// A minimal C++ example: build a market context, then ask the same question
// against the stored context and against the context passed inline -- both
// through the C++ hull.
//
// This goes through `wickra_copilot.hpp`, the C++ hull shipped beside the C
// header, because that hull is what a C++ caller is meant to use: it owns and
// frees the handle, runs the two-call length protocol behind
// `wickra_copilot_command` for you -- the core carries the produced-but-
// undelivered response between the two calls, so a mutating `build_context`
// runs once, not twice -- and turns a refusal into an exception rather than a
// negative integer that is easy to ignore. Calling the C functions directly
// from C++ works too, but then the hull would be shipped without anything
// building it.
#include <cstdio>
#include <string>

#include "wickra_copilot.hpp"

namespace {
const char *SPEC =
    R"({"symbols":["BTCUSDT"],"lookback":3,"facts":["price_move"]})";

// A three-bar BTC dump (100 -> 94) fed inline as a build_context command.
const char *BUILD =
    R"({"cmd":"build_context","feeds":{"BTCUSDT":{"symbol":"BTCUSDT","candles":[)"
    R"({"ts":1,"open":100,"high":100,"low":100,"close":100,"volume":1},)"
    R"({"ts":2,"open":97,"high":97,"low":97,"close":97,"volume":1},)"
    R"({"ts":3,"open":94,"high":94,"low":94,"close":94,"volume":1}]}}})";

const char *QUESTION = "why did BTC dump";
}  // namespace

int main() {
    try {
        std::printf("wickra-copilot %s\n", wickra::Copilot::version().c_str());

        // Build once; the handle stores the context for later queries.
        wickra::Copilot copilot(SPEC);
        const std::string context = copilot.command(BUILD);
        std::printf("context: %s\n", context.c_str());
        const std::string from_stored =
            copilot.command(std::string(R"({"cmd":"query","question":")") + QUESTION + "\"}");

        // A fresh handle never builds; the context travels inline with the query.
        wickra::Copilot fresh(SPEC);
        const std::string from_inline = fresh.command(
            std::string(R"({"cmd":"query","question":")") + QUESTION + R"(","context":)" + context + "}");
        std::printf("tool_calls: %s\n", from_stored.c_str());

        // Both ways of carrying the context answer the same.
        if (from_stored != from_inline) {
            std::fprintf(stderr, "inline and stored contexts disagree\n");
            return 1;
        }
    } catch (const wickra::CopilotError &err) {
        // Every failure arrives here: a spec the core rejects, a command it does
        // not understand, a call that returned a negative code.
        std::fprintf(stderr, "%s\n", err.what());
        return 1;
    }
    return 0;
}
