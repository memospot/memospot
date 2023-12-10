// Dependencies are managed with udd
// See: https://github.com/hayd/deno-udd
//
// Update behavior:
// ^ | Compatible     | major version is the same (if major=0 then same minor version)
// ~ | Approximately  |	major and minor version are the same (or both major=0)
// < | Less than      | less than the provided version
// = | Equal          | is exactly this version

export {
    assert,
    assertEquals,
    assertIsError,
    assertStringIncludes,
    assertThrows,
} from "https://deno.land/std@0.208.0/assert/mod.ts#^";

export { crypto } from "https://deno.land/std@0.208.0/crypto/mod.ts#^";
export { encodeHex } from "https://deno.land/std@0.208.0/encoding/hex.ts#^";
export { existsSync } from "https://deno.land/std@0.208.0/fs/mod.ts#^";
export { isAbsolute } from "https://deno.land/std@0.208.0/path/mod.ts#^";

import osPaths from "https://deno.land/x/os_paths@v7.4.0/src/mod.deno.ts#^";
export { osPaths };

// @deno-types="https://cdn.skypack.dev/fflate@0.8.1/lib/index.d.ts#^"
// export * as fflate from "https://cdn.skypack.dev/fflate@0.8.0?min#^";