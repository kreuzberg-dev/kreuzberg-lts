package kreuzberg

/*
// Kreuzberg FFI - Static Linking Configuration
//
// The library links statically to avoid runtime dependency issues.
// Binaries built with this package are self-contained.
//
// Build locations searched (in order):
//   1. Development: ${SRCDIR}/../../../target/release/ (monorepo builds)
//   2. pkg-config: kreuzberg-ffi (if PKG_CONFIG_PATH is set)
//
// For users installing via go get:
//   Download pre-built library from GitHub releases and set CGO_LDFLAGS.
//   See README.md for detailed instructions.

// macOS: Direct path to static library (Apple ld does not support -Bstatic)
#cgo darwin CFLAGS: -I${SRCDIR}/internal/ffi
#cgo darwin LDFLAGS: ${SRCDIR}/../../../target/release/libkreuzberg_ffi.a -framework CoreFoundation -framework CoreServices -framework SystemConfiguration -framework Security -lc++

// Linux: Use GNU ld static/dynamic switching
#cgo linux CFLAGS: -I${SRCDIR}/internal/ffi
#cgo linux LDFLAGS: -L${SRCDIR}/../../../target/release -Wl,-Bstatic -lkreuzberg_ffi -Wl,-Bdynamic -lpthread -ldl -lm -lstdc++

// Windows: Static library with Windows system libs
#cgo windows CFLAGS: -I${SRCDIR}/internal/ffi
#cgo windows LDFLAGS: -L${SRCDIR}/../../../target/release -lkreuzberg_ffi -lws2_32 -luserenv -lbcrypt -lntdll -static-libgcc -static-libstdc++

#include "internal/ffi/kreuzberg.h"
#include <stdlib.h>
#include <stdint.h>
*/
import "C"
