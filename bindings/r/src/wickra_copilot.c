/* R .Call glue for the wickra-copilot C ABI hub. */
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>
#include <stddef.h>
#include "wickra_copilot.h"

/* --- handle lifetime ----------------------------------------------------- */

static void wkcopilot_finalize(SEXP ext) {
    WickraCopilot *h = (WickraCopilot *)R_ExternalPtrAddr(ext);
    if (h) {
        wickra_copilot_free(h);
    }
    R_ClearExternalPtr(ext);
}

static WickraCopilot *handle_of(SEXP ext) {
    WickraCopilot *h = (WickraCopilot *)R_ExternalPtrAddr(ext);
    if (!h) {
        Rf_error("wickra-copilot: handle is closed");
    }
    return h;
}

/* --- exported .Call entries ---------------------------------------------- */

SEXP wkcopilot_version(void) {
    return Rf_mkString(wickra_copilot_version());
}

SEXP wkcopilot_new(SEXP spec_json) {
    WickraCopilot *h = wickra_copilot_new(CHAR(STRING_ELT(spec_json, 0)));
    if (!h) {
        Rf_error("wickra-copilot: invalid spec");
    }
    SEXP ext = PROTECT(R_MakeExternalPtr(h, R_NilValue, R_NilValue));
    R_RegisterCFinalizerEx(ext, wkcopilot_finalize, TRUE);
    UNPROTECT(1);
    return ext;
}

SEXP wkcopilot_command(SEXP ext, SEXP cmd_json) {
    WickraCopilot *h = handle_of(ext);
    const char *cmd = CHAR(STRING_ELT(cmd_json, 0));

    /* Length-out protocol: learn the length, then read into a caller buffer.
       Domain errors come back in-band as {"ok":false,...} JSON, not a negative
       code; only unusable arguments / a caught panic return < 0. */
    int len = wickra_copilot_command(h, cmd, NULL, 0);
    if (len < 0) {
        Rf_error("wickra-copilot: command failed (code %d)", len);
    }
    char *buf = (char *)R_alloc((size_t)len + 1, 1);
    wickra_copilot_command(h, cmd, buf, (size_t)len + 1);
    return Rf_mkString(buf);
}

/* --- registration -------------------------------------------------------- */

static const R_CallMethodDef CallEntries[] = {
    {"wkcopilot_version", (DL_FUNC)&wkcopilot_version, 0},
    {"wkcopilot_new", (DL_FUNC)&wkcopilot_new, 1},
    {"wkcopilot_command", (DL_FUNC)&wkcopilot_command, 2},
    {NULL, NULL, 0}};

void R_init_wickracopilot(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}
