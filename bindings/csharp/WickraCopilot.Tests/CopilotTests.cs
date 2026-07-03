using System.Text.Json;
using Wickra.Copilot;
using Xunit;

namespace WickraCopilot.Tests;

public class CopilotTests
{
    private const string Spec =
        "{\"symbols\":[\"BTCUSDT\"],\"lookback\":3,\"facts\":[\"price_move\"]}";

    private static object Candle(long ts, double close) =>
        new { ts, open = close, high = close, low = close, close, volume = 1.0 };

    private static string BuildContext() => JsonSerializer.Serialize(new
    {
        cmd = "build_context",
        feeds = new Dictionary<string, object>
        {
            ["BTCUSDT"] = new
            {
                symbol = "BTCUSDT",
                candles = new[] { Candle(1, 100.0), Candle(2, 97.0), Candle(3, 94.0) },
            },
        },
    });

    [Fact]
    public void Version_IsNonEmpty()
    {
        Assert.False(string.IsNullOrEmpty(Copilot.Version()));
    }

    [Fact]
    public void BuildContext_Roundtrip()
    {
        using var copilot = new Copilot(Spec);
        string raw = copilot.Command(BuildContext());
        using JsonDocument ctx = JsonDocument.Parse(raw);

        JsonElement facts = ctx.RootElement.GetProperty("facts");
        Assert.Equal(1, facts.GetArrayLength());
        Assert.Equal("price_move", facts[0].GetProperty("kind").GetString());
        // BTC drops 6% over three bars.
        Assert.Equal(-6.0, facts[0].GetProperty("value").GetDouble(), 9);
    }

    [Fact]
    public void Query_RoutesToPriceMove()
    {
        using var copilot = new Copilot(Spec);
        copilot.Command(BuildContext());
        string raw = copilot.Command("{\"cmd\":\"query\",\"question\":\"why did BTC dump\"}");
        using JsonDocument result = JsonDocument.Parse(raw);

        bool hasPriceMove = false;
        foreach (JsonElement call in result.RootElement.GetProperty("tool_calls").EnumerateArray())
        {
            if (call.GetProperty("kind").GetString() == "price_move")
            {
                hasPriceMove = true;
            }
        }
        Assert.True(hasPriceMove);
    }

    [Fact]
    public void InvalidSpec_Throws()
    {
        Assert.Throws<ArgumentException>(() => new Copilot("not json"));
    }

    [Fact]
    public void UnknownCommand_IsInBandError()
    {
        using var copilot = new Copilot(Spec);
        // An unknown command is not a hard error: the ABI returns a length and the
        // error surfaces in-band as {"ok":false,...} JSON.
        string raw = copilot.Command("{\"cmd\":\"nope\"}");
        Assert.Contains("\"ok\":false", raw);
    }
}
