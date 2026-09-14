package copilot

// Operating-mode equivalence through the binding, over the golden corpus. The
// command protocol offers two ways to reach the same answer, and both must
// return the same bytes: `facts` is an alias of `build_context`, and `query`
// answers either against the context the handle stored from its last
// `build_context` or against a context passed inline. The core pins this in
// Rust (operating_modes.rs); this checks the boundary the Go binding crosses.
// A missing corpus is a failure, not a skip.

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

const modesQuestion = "what moved and why?"

func command(t *testing.T, c *Copilot, cmd map[string]any) string {
	t.Helper()
	b, err := json.Marshal(cmd)
	if err != nil {
		t.Fatal(err)
	}
	out, err := c.Command(string(b))
	if err != nil {
		t.Fatal(err)
	}
	return strings.TrimSpace(out)
}

func TestFactsAliasAndInlineQueryMatchTheStoredPath(t *testing.T) {
	g := goldenDir()
	if g == "" {
		t.Fatal("golden corpus not found")
	}
	feeds, err := os.ReadFile(filepath.Join(g, "feeds.json"))
	if err != nil {
		t.Fatal(err)
	}
	specs, err := filepath.Glob(filepath.Join(g, "specs", "*.json"))
	if err != nil || len(specs) == 0 {
		t.Fatal("golden corpus not found")
	}
	sort.Strings(specs)
	for _, specPath := range specs {
		name := filepath.Base(specPath)
		spec, err := os.ReadFile(specPath)
		if err != nil {
			t.Fatal(err)
		}
		expected, err := os.ReadFile(filepath.Join(g, "expected", name))
		if err != nil {
			t.Fatal(err)
		}

		stored, err := New(string(spec))
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		built := command(t, stored, map[string]any{"cmd": "build_context", "feeds": json.RawMessage(feeds)})
		if built != strings.TrimSpace(string(expected)) {
			t.Fatalf("%s: build_context does not match the blessed context", name)
		}
		fromStored := command(t, stored, map[string]any{"cmd": "query", "question": modesQuestion})
		stored.Close()

		fresh, err := New(string(spec))
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		fromInline := command(t, fresh, map[string]any{"cmd": "query", "question": modesQuestion, "context": json.RawMessage(built)})
		fresh.Close()
		if fromInline != fromStored {
			t.Fatalf("%s: an inline context answers differently from the stored one", name)
		}

		alias, err := New(string(spec))
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		viaFacts := command(t, alias, map[string]any{"cmd": "facts", "feeds": json.RawMessage(feeds)})
		alias.Close()
		if viaFacts != built {
			t.Fatalf("%s: facts differs from build_context", name)
		}
	}
}
