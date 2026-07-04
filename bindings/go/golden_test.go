package copilot

// Cross-language golden parity: build the copilot from each committed
// golden/specs/*.json, run build_context over the shared golden/feeds.json and
// read back the context, then assert it equals golden/expected/<spec>.json
// byte-for-byte. The binding returns the core's compact command_json string
// verbatim, so byte equality is the exact cross-language parity check.

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func goldenDir() string {
	dir, err := os.Getwd()
	if err != nil {
		return ""
	}
	for i := 0; i < 8; i++ {
		g := filepath.Join(dir, "golden")
		if _, err := os.Stat(filepath.Join(g, "specs")); err == nil {
			return g
		}
		dir = filepath.Dir(dir)
	}
	return ""
}

func TestGoldenParity(t *testing.T) {
	g := goldenDir()
	if g == "" {
		t.Skip("golden fixtures not present")
	}
	feeds, err := os.ReadFile(filepath.Join(g, "feeds.json"))
	if err != nil {
		t.Fatal(err)
	}
	specs, err := filepath.Glob(filepath.Join(g, "specs", "*.json"))
	if err != nil {
		t.Fatal(err)
	}
	for _, specPath := range specs {
		specJSON, err := os.ReadFile(specPath)
		if err != nil {
			t.Fatal(err)
		}
		name := filepath.Base(specPath)
		expected, err := os.ReadFile(filepath.Join(g, "expected", name))
		if err != nil {
			t.Fatal(err)
		}
		c, err := New(string(specJSON))
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		build, err := json.Marshal(map[string]any{"cmd": "build_context", "feeds": json.RawMessage(feeds)})
		if err != nil {
			c.Close()
			t.Fatal(err)
		}
		raw, err := c.Command(string(build))
		c.Close()
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		// The blessed file carries a trailing newline from the CLI's println; the
		// command reply does not. Trim both so the comparison is exact.
		if strings.TrimSpace(raw) != strings.TrimSpace(string(expected)) {
			t.Fatalf("%s: golden mismatch", name)
		}
	}
}
