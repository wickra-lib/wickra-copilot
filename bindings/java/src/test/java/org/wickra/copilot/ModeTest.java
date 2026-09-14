package org.wickra.copilot;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;

// Operating-mode equivalence through the binding, over the golden corpus. The
// command protocol offers two ways to reach the same answer, and both must
// return the same bytes: `facts` is an alias of `build_context`, and `query`
// answers either against the context the handle stored from its last
// `build_context` or against a context passed inline. The core pins this in
// Rust (operating_modes.rs); this checks the boundary the Java binding
// crosses. A missing corpus is a failure, not a skip. The command JSON is
// assembled as text: the context the core returned is spliced in verbatim.
class ModeTest {
    private static final String QUESTION = "what moved and why?";

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
    void factsAliasAndInlineQueryMatchTheStoredPath() throws IOException {
        Path golden = findGolden();
        assertNotNull(golden, "golden corpus not found");

        String feeds = Files.readString(golden.resolve("feeds.json")).strip();
        List<Path> specPaths;
        try (Stream<Path> specs = Files.list(golden.resolve("specs"))) {
            specPaths = specs.filter(p -> p.toString().endsWith(".json")).sorted().toList();
        }
        assertFalse(specPaths.isEmpty(), "golden corpus not found");

        for (Path specPath : specPaths) {
            String spec = Files.readString(specPath);
            String name = specPath.getFileName().toString();
            String expected = Files.readString(golden.resolve("expected").resolve(name)).strip();

            String built;
            String fromStored;
            try (Copilot stored = new Copilot(spec)) {
                built = stored.command("{\"cmd\":\"build_context\",\"feeds\":" + feeds + "}").strip();
                assertEquals(expected, built, name);
                fromStored = stored.command("{\"cmd\":\"query\",\"question\":\"" + QUESTION + "\"}").strip();
            }
            try (Copilot fresh = new Copilot(spec)) {
                String fromInline = fresh.command(
                        "{\"cmd\":\"query\",\"question\":\"" + QUESTION + "\",\"context\":" + built + "}").strip();
                assertEquals(fromStored, fromInline, name + ": an inline context answers differently from the stored one");
            }
            try (Copilot alias = new Copilot(spec)) {
                String viaFacts = alias.command("{\"cmd\":\"facts\",\"feeds\":" + feeds + "}").strip();
                assertEquals(built, viaFacts, name + ": facts differs from build_context");
            }
        }
    }
}
