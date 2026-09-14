/* Cross-language golden parity and operating-mode equivalence, from C.
 *
 * Golden: build the copilot from each committed golden/specs/*.json, run
 * build_context over the shared golden/feeds.json and assert the context
 * equals golden/expected/<spec>.json byte-for-byte. The ABI returns the
 * core's compact command output verbatim, so byte equality is the exact
 * cross-language parity check -- the same one Python, Node, Go, C#, Java, R
 * and WASM make.
 *
 * Operating mode: the command protocol offers two ways to reach the same
 * answer -- `facts` is an alias of `build_context`, and `query` answers either
 * against the context the handle stored from its last build or against a
 * context passed inline -- and both must return the same bytes. The core pins
 * this in Rust (operating_modes.rs); this checks the boundary six of the ten
 * language reaches cross. C has no JSON library, so the command JSON is
 * assembled as text with the context the core returned spliced in verbatim.
 *
 * C has no directory API that is portable between POSIX and Windows, so the
 * spec list is globbed by CMake at configure time and written into
 * golden_specs.h. That keeps the property the other bindings get from a
 * runtime glob: a spec added to the corpus is covered here without editing
 * this file.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "golden_specs.h" /* GOLDEN_DIR, GOLDEN_SPECS, GOLDEN_SPEC_COUNT */
#include "wickra_copilot.h"

/* Read a whole file. Caller frees. Returns NULL and reports on failure. */
static char *slurp(const char *path) {
    FILE *file = fopen(path, "rb");
    if (!file) {
        fprintf(stderr, "cannot open %s\n", path);
        return NULL;
    }
    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        return NULL;
    }
    long size = ftell(file);
    if (size < 0 || fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        return NULL;
    }
    char *buf = (char *)malloc((size_t)size + 1);
    if (!buf) {
        fclose(file);
        return NULL;
    }
    size_t got = fread(buf, 1, (size_t)size, file);
    fclose(file);
    buf[got] = '\0';
    return buf;
}

/* Trim ASCII whitespace in place and return the start of the trimmed text. */
static char *trim(char *text) {
    while (*text == ' ' || *text == '\n' || *text == '\r' || *text == '\t') {
        text++;
    }
    size_t len = strlen(text);
    while (len > 0) {
        char last = text[len - 1];
        if (last != ' ' && last != '\n' && last != '\r' && last != '\t') {
            break;
        }
        text[--len] = '\0';
    }
    return text;
}

/* A growable string. */
typedef struct {
    char *buf;
    size_t len;
    size_t cap;
} Str;

static int str_push(Str *s, const char *text, size_t n) {
    if (s->len + n + 1 > s->cap) {
        size_t cap = s->cap ? s->cap : 4096;
        while (cap < s->len + n + 1) {
            cap *= 2;
        }
        char *grown = (char *)realloc(s->buf, cap);
        if (!grown) {
            return 0;
        }
        s->buf = grown;
        s->cap = cap;
    }
    memcpy(s->buf + s->len, text, n);
    s->len += n;
    s->buf[s->len] = '\0';
    return 1;
}

static int str_puts(Str *s, const char *text) { return str_push(s, text, strlen(text)); }

/* Apply one read-only command through the two-call length protocol. Caller
 * frees. The hub caches the response of a mutating command between the length
 * call and the delivering call, so build_context runs once, not twice. */
static char *run(WickraCopilot *copilot, const char *cmd) {
    int32_t len = wickra_copilot_command(copilot, cmd, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed with code %d\n", (int)len);
        return NULL;
    }
    char *out = (char *)malloc((size_t)len + 1);
    if (!out) {
        return NULL;
    }
    if (wickra_copilot_command(copilot, cmd, out, (size_t)len + 1) < 0) {
        free(out);
        return NULL;
    }
    return out;
}

/* `{"cmd":<verb>,"feeds":<feeds>}` on a fresh handle; returns the reply and,
 * through `handle_out`, the handle so a later `query` can read what it stored.
 * Caller frees both. */
static char *build(const char *spec, const char *verb, const char *feeds, WickraCopilot **handle_out) {
    WickraCopilot *copilot = wickra_copilot_new(spec);
    if (!copilot) {
        fprintf(stderr, "invalid spec\n");
        return NULL;
    }
    Str cmd = {0};
    if (!str_puts(&cmd, "{\"cmd\":\"") || !str_puts(&cmd, verb) || !str_puts(&cmd, "\",\"feeds\":") ||
        !str_puts(&cmd, feeds) || !str_puts(&cmd, "}")) {
        wickra_copilot_free(copilot);
        return NULL;
    }
    char *out = run(copilot, cmd.buf);
    free(cmd.buf);
    if (!out) {
        wickra_copilot_free(copilot);
        return NULL;
    }
    *handle_out = copilot;
    return out;
}

/* `{"cmd":"query","question":...}` against what the handle stored, or with the
 * context spliced in when `context` is given. Caller frees. */
static char *query(WickraCopilot *copilot, const char *context) {
    Str cmd = {0};
    int ok = str_puts(&cmd, "{\"cmd\":\"query\",\"question\":\"what moved and why?\"");
    if (ok && context) {
        ok = str_puts(&cmd, ",\"context\":") && str_puts(&cmd, context);
    }
    if (!ok || !str_puts(&cmd, "}")) {
        free(cmd.buf);
        return NULL;
    }
    char *out = run(copilot, cmd.buf);
    free(cmd.buf);
    return out;
}

int main(void) {
    printf("wickra-copilot %s: golden parity + operating modes over %zu spec(s)\n",
           wickra_copilot_version(), (size_t)GOLDEN_SPEC_COUNT);
    if (GOLDEN_SPEC_COUNT == 0) {
        fprintf(stderr, "golden corpus not found\n");
        return 1;
    }
    char *feeds = slurp(GOLDEN_DIR "/feeds.json");
    if (!feeds) {
        return 1;
    }
    int failures = 0;
    for (size_t i = 0; GOLDEN_SPECS[i]; i++) {
        char spec_path[1024];
        char expected_path[1024];
        snprintf(spec_path, sizeof spec_path, "%s/specs/%s", GOLDEN_DIR, GOLDEN_SPECS[i]);
        snprintf(expected_path, sizeof expected_path, "%s/expected/%s", GOLDEN_DIR, GOLDEN_SPECS[i]);
        char *spec = slurp(spec_path);
        char *expected_raw = slurp(expected_path);
        if (!spec || !expected_raw) {
            free(spec);
            free(expected_raw);
            failures++;
            continue;
        }
        char *expected = trim(expected_raw);

        WickraCopilot *stored = NULL;
        WickraCopilot *alias = NULL;
        WickraCopilot *fresh = wickra_copilot_new(spec);
        char *built = build(spec, "build_context", feeds, &stored);
        char *via_facts = build(spec, "facts", feeds, &alias);
        char *from_stored = stored ? query(stored, NULL) : NULL;
        char *from_inline = (fresh && built) ? query(fresh, trim(built)) : NULL;
        if (!built || !via_facts || !from_stored || !from_inline) {
            fprintf(stderr, "%s: command failed\n", GOLDEN_SPECS[i]);
            failures++;
        } else if (strcmp(trim(built), expected) != 0) {
            fprintf(stderr, "%s: build_context does not match the blessed context\n", GOLDEN_SPECS[i]);
            failures++;
        } else if (strcmp(trim(via_facts), trim(built)) != 0) {
            fprintf(stderr, "%s: facts differs from build_context\n", GOLDEN_SPECS[i]);
            failures++;
        } else if (strcmp(trim(from_inline), trim(from_stored)) != 0) {
            fprintf(stderr, "%s: an inline context answers differently from the stored one\n", GOLDEN_SPECS[i]);
            failures++;
        } else {
            printf("  %s: ok\n", GOLDEN_SPECS[i]);
        }
        free(built);
        free(via_facts);
        free(from_stored);
        free(from_inline);
        wickra_copilot_free(stored);
        wickra_copilot_free(alias);
        wickra_copilot_free(fresh);
        free(spec);
        free(expected_raw);
    }
    free(feeds);
    if (failures) {
        fprintf(stderr, "%d spec(s) failed\n", failures);
        return 1;
    }
    printf("ok\n");
    return 0;
}
