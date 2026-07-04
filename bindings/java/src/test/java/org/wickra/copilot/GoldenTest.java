package org.wickra.copilot;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;

// Cross-language golden parity: build the copilot from each committed
// golden/specs/*.json, run build_context over the shared golden/feeds.json and
// read back the context, then assert it equals golden/expected/<spec>.json
// byte-for-byte. The binding returns the core's compact command_json string
// verbatim, so byte equality is the exact cross-language parity check.
class GoldenTest {
    private static Path findGolden() {
        Path dir = Path.of("").toAbsolutePath();
        for (int i = 0; i < 8 && dir != null; i++) {
            Path g = dir.resolve("golden");
            if (Files.isDirectory(g.resolve("specs"))) {
                return g;
            }
            dir = dir.getParent();
        }
        return null;
    }

    @Test
    void goldenContextsAreByteIdentical() throws IOException {
        Path golden = findGolden();
        assumeTrue(golden != null, "golden fixtures not present");

        String feeds = Files.readString(golden.resolve("feeds.json")).strip();
        String build = "{\"cmd\":\"build_context\",\"feeds\":" + feeds + "}";
        try (Stream<Path> specs = Files.list(golden.resolve("specs"))) {
            for (Path specPath : specs.filter(p -> p.toString().endsWith(".json")).toList()) {
                String spec = Files.readString(specPath);
                String name = specPath.getFileName().toString();
                String expected = Files.readString(golden.resolve("expected").resolve(name)).strip();
                try (Copilot copilot = new Copilot(spec)) {
                    String raw = copilot.command(build);
                    assertEquals(expected, raw.strip(), name);
                }
            }
        }
    }
}
