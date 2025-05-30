// Copyright 2016-2024 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use super::Neon;

// Work around a bug in LLVM/rustc where `-C target_cpu=cortex-a72`--
// and `-C target_cpu=native` on Cortex-A72 Raspberry PI devices in
// particular--enables crypto features even though not all Cortex-A72
// CPUs have crypto features:
//
// ```
// $ rustc --print cfg --target=aarch64-unknown-linux-gnu | grep feature
// target_feature="neon"
// $ rustc --print cfg --target=aarch64-unknown-linux-gnu -C target_cpu=cortex-a72 | grep feature
// target_feature="aes"
// target_feature="crc"
// target_feature="neon"
// target_feature="pmuv3"
// target_feature="sha2"
// ```
//
// XXX/TODO(MSRV https://github.com/llvm/llvm-project/issues/90365): This
// workaround is heavy-handed since it forces extra branches for devices that
// have correctly-modeled feature sets, so it should be removed.
pub const FORCE_DYNAMIC_DETECTION: u32 = !Neon::mask();

pub fn detect_features() -> u32 {
    let mut features = 0;

    // When linked statically, uclibc doesn't provide getauxval. When linked
    // dynamically, recent versions do provide it, but we want to support older
    // versions too. Assume that if uclibc is being used, this is an embedded
    // target where the user cares a lot about minimizing code size and also
    // that they know in advance exactly what target features are supported, so
    // rely only on static feature detection.
    #[cfg(not(target_env = "uclibc"))]
    {
        use super::CAPS_STATIC;
        use libc::{c_ulong, getauxval, AT_HWCAP};

        const HWCAP_NEON: c_ulong = 1 << 12;

        if CAPS_STATIC & Neon::mask() != Neon::mask() {
            let caps = unsafe { getauxval(AT_HWCAP) };

            // OpenSSL and BoringSSL don't enable any other features if NEON isn't
            // available. We don't enable any hardware implementations for 32-bit ARM.
            if caps & HWCAP_NEON == HWCAP_NEON {
                features |= Neon::mask();
            }
        }
    }

    features
}
