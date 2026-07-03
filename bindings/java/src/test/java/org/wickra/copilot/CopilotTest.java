package org.wickra.copilot;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class CopilotTest {
    private static final String SPEC =
            "{\"symbols\":[\"BTCUSDT\"],\"lookback\":3,\"facts\":[\"price_move\"]}";

    private static String candle(int ts, String close) {
        return "{\"ts\":" + ts + ",\"open\":" + close + ",\"high\":" + close
                + ",\"low\":" + close + ",\"close\":" + close + ",\"volume\":1.0}";
    }

    private static String buildContext() {
        // BTC drops 6% over three bars -> one significant price-move fact.
        return "{\"cmd\":\"build_context\",\"feeds\":{\"BTCUSDT\":{\"symbol\":\"BTCUSDT\",\"candles\":["
                + candle(1, "100.0") + "," + candle(2, "97.0") + "," + candle(3, "94.0") + "]}}}";
    }

    @Test
    void versionIsNonEmpty() {
        assertFalse(Copilot.version().isEmpty());
    }

    @Test
    void buildContextRoundtrip() {
        try (Copilot copilot = new Copilot(SPEC)) {
            String raw = copilot.command(buildContext());
            assertTrue(raw.contains("\"price_move\""), raw);
            assertTrue(raw.contains("\"symbol\":\"BTCUSDT\""), raw);
        }
    }

    @Test
    void queryRoutesToPriceMove() {
        try (Copilot copilot = new Copilot(SPEC)) {
            copilot.command(buildContext());
            String raw = copilot.command("{\"cmd\":\"query\",\"question\":\"why did BTC dump\"}");
            assertTrue(raw.contains("\"tool_calls\""), raw);
            assertTrue(raw.contains("\"price_move\""), raw);
        }
    }

    @Test
    void invalidSpecThrows() {
        assertThrows(IllegalArgumentException.class, () -> new Copilot("not json"));
    }

    @Test
    void unknownCommandIsInBandError() {
        try (Copilot copilot = new Copilot(SPEC)) {
            // An unknown command is not a hard error: the ABI returns a length and
            // the error surfaces in-band as {"ok":false,...} JSON.
            String raw = copilot.command("{\"cmd\":\"nope\"}");
            assertTrue(raw.contains("\"ok\":false"), raw);
        }
    }
}
