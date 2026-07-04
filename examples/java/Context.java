// A runnable Java example: build a market context through the binding.
//
//   cargo build -p wickra-copilot-c
//   mvn -f bindings/java/pom.xml -q package -DskipTests
//   javac -cp bindings/java/target/classes examples/java/Context.java -d examples/java/out
//   java --enable-native-access=ALL-UNNAMED \
//        -Dnative.lib.dir=target/debug \
//        -cp "bindings/java/target/classes;examples/java/out" Context
import org.wickra.copilot.Copilot;

public final class Context {
    private static final String SPEC =
            "{\"symbols\":[\"BTCUSDT\"],\"lookback\":3,\"facts\":[\"price_move\"]}";

    private static final String BUILD =
            "{\"cmd\":\"build_context\",\"feeds\":{\"BTCUSDT\":{\"symbol\":\"BTCUSDT\",\"candles\":["
                    + "{\"ts\":1,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1},"
                    + "{\"ts\":2,\"open\":97,\"high\":97,\"low\":97,\"close\":97,\"volume\":1},"
                    + "{\"ts\":3,\"open\":94,\"high\":94,\"low\":94,\"close\":94,\"volume\":1}]}}}";

    public static void main(String[] args) {
        try (Copilot copilot = new Copilot(SPEC)) {
            String response = copilot.command(BUILD);
            System.out.println("wickra-copilot " + Copilot.version());
            System.out.println(response);
        }
    }
}
