using System.Text.Json;
using Wickra.Copilot;
using Xunit;

namespace WickraCopilot.Tests;

// Cross-language golden parity: build the copilot from each committed
// golden/specs/*.json, run build_context over the shared golden/feeds.json and
// read back the context, then assert it equals golden/expected/<spec>.json
// byte-for-byte. The binding returns the core's compact command_json string
// verbatim, so byte equality is the exact cross-language parity check.
public class GoldenTests
{
    private static string? FindGolden()
    {
        string? dir = AppContext.BaseDirectory;
        for (int i = 0; i < 10 && dir is not null; i++)
        {
            string g = Path.Combine(dir, "golden");
            if (Directory.Exists(Path.Combine(g, "specs")))
            {
                return g;
            }
            dir = Path.GetDirectoryName(dir);
        }
        return null;
    }

    [Fact]
    public void GoldenContexts_AreByteIdentical()
    {
        string? golden = FindGolden();
        if (golden is null)
        {
            return; // golden fixtures not present
        }

        string feedsJson = File.ReadAllText(Path.Combine(golden, "feeds.json"));
        using JsonDocument feeds = JsonDocument.Parse(feedsJson);

        foreach (string specPath in Directory.GetFiles(Path.Combine(golden, "specs"), "*.json"))
        {
            string spec = File.ReadAllText(specPath);
            string name = Path.GetFileName(specPath);
            string expected = File.ReadAllText(Path.Combine(golden, "expected", name)).TrimEnd();

            using var copilot = new Copilot(spec);
            string build = JsonSerializer.Serialize(new { cmd = "build_context", feeds = feeds.RootElement });
            string raw = copilot.Command(build);
            Assert.Equal(expected, raw.TrimEnd());
        }
    }
}
