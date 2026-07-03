// Package copilot provides idiomatic Go bindings for wickra-copilot over its C
// ABI hub: build a Copilot from a spec JSON, drive it with command JSON and read
// back the response JSON — the same protocol as the CLI and every other binding.
// Only the deterministic core is exposed; the LLM adapter is never reachable
// over the C ABI, so the network and API key stay off this surface.
//
// The binding links the prebuilt C ABI library, staged per platform under
// ./lib/<goos>_<goarch>/, with the header vendored under ./include.
package copilot

/*
#cgo CFLAGS: -I${SRCDIR}/include
#cgo linux,amd64 LDFLAGS: -L${SRCDIR}/lib/linux_amd64 -lwickra_copilot -Wl,-rpath,${SRCDIR}/lib/linux_amd64
#cgo linux,arm64 LDFLAGS: -L${SRCDIR}/lib/linux_arm64 -lwickra_copilot -Wl,-rpath,${SRCDIR}/lib/linux_arm64
#cgo darwin,amd64 LDFLAGS: -L${SRCDIR}/lib/darwin_amd64 -lwickra_copilot -Wl,-rpath,${SRCDIR}/lib/darwin_amd64
#cgo darwin,arm64 LDFLAGS: -L${SRCDIR}/lib/darwin_arm64 -lwickra_copilot -Wl,-rpath,${SRCDIR}/lib/darwin_arm64
#cgo windows,amd64 LDFLAGS: -L${SRCDIR}/lib/windows_amd64 -l:wickra_copilot.dll
#cgo windows,arm64 LDFLAGS: -L${SRCDIR}/lib/windows_arm64 -l:wickra_copilot.dll
#include <stdlib.h>
#include "wickra_copilot.h"
*/
import "C"

import (
	"fmt"
	"runtime"
	"unsafe"
)

// Copilot is a copilot instance driven by JSON commands.
type Copilot struct {
	handle *C.WickraCopilot
}

// New builds a copilot from a spec JSON string. Call Close when done (a
// finalizer also frees it, but explicit Close is preferred).
func New(specJSON string) (*Copilot, error) {
	cspec := C.CString(specJSON)
	defer C.free(unsafe.Pointer(cspec))

	handle := C.wickra_copilot_new(cspec)
	if handle == nil {
		return nil, fmt.Errorf("wickra-copilot: invalid spec")
	}
	c := &Copilot{handle: handle}
	runtime.SetFinalizer(c, (*Copilot).Close)
	return c, nil
}

// Command applies a command JSON and returns the response JSON. It uses the C
// ABI's length-out protocol: a first call learns the length, then the response
// is read into a caller-owned buffer.
func (c *Copilot) Command(cmdJSON string) (string, error) {
	ccmd := C.CString(cmdJSON)
	defer C.free(unsafe.Pointer(ccmd))

	n := C.wickra_copilot_command(c.handle, ccmd, nil, 0)
	if n < 0 {
		return "", fmt.Errorf("wickra-copilot: command failed (code %d)", int(n))
	}
	buf := make([]byte, int(n)+1)
	C.wickra_copilot_command(
		c.handle,
		ccmd,
		(*C.char)(unsafe.Pointer(&buf[0])),
		C.size_t(len(buf)),
	)
	return string(buf[:n]), nil
}

// Close frees the copilot handle. Safe to call more than once.
func (c *Copilot) Close() {
	if c.handle != nil {
		C.wickra_copilot_free(c.handle)
		c.handle = nil
	}
	runtime.SetFinalizer(c, nil)
}

// Version returns the library version.
func Version() string {
	return C.GoString(C.wickra_copilot_version())
}
