// A runnable .NET example: build a market context through the binding.
//
//   cargo build --release -p wickra-copilot-c
//   dotnet run --project examples/csharp/Context

using System.Text.Json;
using Wickra.Copilot;

const string spec =
    "{\"symbols\":[\"BTCUSDT\"],\"lookback\":3,\"facts\":[\"price_move\"]}";

const string build =
    "{\"cmd\":\"build_context\",\"feeds\":{\"BTCUSDT\":{\"symbol\":\"BTCUSDT\",\"candles\":[" +
    "{\"ts\":1,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1}," +
    "{\"ts\":2,\"open\":97,\"high\":97,\"low\":97,\"close\":97,\"volume\":1}," +
    "{\"ts\":3,\"open\":94,\"high\":94,\"low\":94,\"close\":94,\"volume\":1}]}}}";

using var copilot = new Copilot(spec);
string response = copilot.Command(build);
using JsonDocument context = JsonDocument.Parse(response);

Console.WriteLine($"wickra-copilot {Copilot.Version()}");
Console.WriteLine(response);
Console.WriteLine($"  facts: {context.RootElement.GetProperty("facts").GetArrayLength()}");
