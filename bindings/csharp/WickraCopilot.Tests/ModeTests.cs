using System.Text.Json;
using Wickra.Copilot;
using Xunit;

namespace WickraCopilot.Tests;

// Operating-mode equivalence through the binding, over the golden corpus. The
// command protocol offers two ways to reach the same answer, and both must
// return the same bytes: `facts` is an alias of `build_context`, and `query`
// answers either against the context the handle stored from its last
// `build_context` or against a context passed inline. The core pins this in
// Rust (operating_modes.rs); this checks the boundary the C# binding crosses.
// A missing corpus is a failure, not a skip.
public class ModeTests
{
    private const string Question = "what moved and why?";

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
    public void FactsAliasAndInlineQuery_MatchTheStoredPath()
    {
        string? golden = FindGolden();
        Assert.NotNull(golden);

        using JsonDocument feeds = JsonDocument.Parse(File.ReadAllText(Path.Combine(golden!, "feeds.json")));
        string[] specs = Directory.GetFiles(Path.Combine(golden!, "specs"), "*.json");
        Array.Sort(specs, StringComparer.Ordinal);
        Assert.NotEmpty(specs);

        foreach (string specPath in specs)
        {
            string spec = File.ReadAllText(specPath);
            string name = Path.GetFileName(specPath);
            string expected = File.ReadAllText(Path.Combine(golden!, "expected", name)).TrimEnd();

            using var stored = new Copilot(spec);
            string built = stored.Command(JsonSerializer.Serialize(new { cmd = "build_context", feeds = feeds.RootElement })).TrimEnd();
            Assert.True(expected == built, $"{name}: build_context does not match the blessed context");
            string fromStored = stored.Command(JsonSerializer.Serialize(new { cmd = "query", question = Question })).TrimEnd();

            using JsonDocument context = JsonDocument.Parse(built);
            using var fresh = new Copilot(spec);
            string fromInline = fresh.Command(JsonSerializer.Serialize(new { cmd = "query", question = Question, context = context.RootElement })).TrimEnd();
            Assert.True(fromStored == fromInline, $"{name}: an inline context answers differently from the stored one");

            using var alias = new Copilot(spec);
            string viaFacts = alias.Command(JsonSerializer.Serialize(new { cmd = "facts", feeds = feeds.RootElement })).TrimEnd();
            Assert.True(built == viaFacts, $"{name}: facts differs from build_context");
        }
    }
}
