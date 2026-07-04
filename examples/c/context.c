/* A minimal C example: build a market context through the wickra-copilot C ABI. */
#include <stdio.h>
#include <stdlib.h>

#include "wickra_copilot.h"

static const char *SPEC =
    "{\"symbols\":[\"BTCUSDT\"],\"lookback\":3,\"facts\":[\"price_move\"]}";

/* A three-bar BTC dump (100 -> 94) fed inline as a build_context command. */
static const char *BUILD =
    "{\"cmd\":\"build_context\",\"feeds\":{\"BTCUSDT\":{\"symbol\":\"BTCUSDT\","
    "\"candles\":["
    "{\"ts\":1,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1},"
    "{\"ts\":2,\"open\":97,\"high\":97,\"low\":97,\"close\":97,\"volume\":1},"
    "{\"ts\":3,\"open\":94,\"high\":94,\"low\":94,\"close\":94,\"volume\":1}]}}}";

/* Length-out protocol: learn the length, then read into a caller buffer.
   Returns a malloc'd NUL-terminated string the caller must free, or NULL. */
static char *run(WickraCopilot *copilot, const char *cmd) {
    int len = wickra_copilot_command(copilot, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    wickra_copilot_command(copilot, cmd, buf, (size_t)len + 1);
    return buf;
}

int main(void) {
    WickraCopilot *copilot = wickra_copilot_new(SPEC);
    if (!copilot) {
        fprintf(stderr, "failed to build copilot\n");
        return 1;
    }

    char *context = run(copilot, BUILD);
    if (!context) {
        wickra_copilot_free(copilot);
        return 1;
    }

    printf("wickra-copilot %s\n", wickra_copilot_version());
    printf("context: %s\n", context);

    free(context);
    wickra_copilot_free(copilot);
    return 0;
}
