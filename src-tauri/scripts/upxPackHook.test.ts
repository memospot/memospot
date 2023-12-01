import {
    assertIsError,
    assertEquals,
} from "https://deno.land/std@0.207.0/assert/mod.ts";

import { upxPackHook } from "./upxPackHook.ts";
import type { UpxOptions } from "./upxPackHook.d.ts";
import { assertStringIncludes } from "https://deno.land/std@0.207.0/assert/assert_string_includes.ts";

Deno.test("upxPackHook", async (t) => {
    await t.step("validate error on unsupported platform", async () => {
        const expected = "`UPX pack` is not supported on";
        let upxOptions: UpxOptions = {
            bin: "upx",
            flags: [],
            fileList: [],
            supportedPlatforms: [],
            ignoreErrors: false,
        };
        {
            const { error } = upxPackHook(upxOptions);
            assertIsError(error, Error, expected);
        }
        {
            upxOptions.ignoreErrors = true;
            const { output, error } = upxPackHook(upxOptions);
            assertEquals(error, null);
            assertStringIncludes(output, expected);
        }
    });

    await t.step("validate error on non-existing file", async () => {
        const expected = "`UPX pack` failed for file `non-existing-file`.";
        let upxOptions: UpxOptions = {
            bin: "upx",
            flags: [],
            fileList: ["non-existing-file"],
            supportedPlatforms: ["windows", "linux", "darwin"],
            ignoreErrors: false,
        };
        {
            const { error } = upxPackHook(upxOptions);
            assertIsError(error, Error, expected);
        }
        {
            upxOptions.ignoreErrors = true;
            const { output, error } = upxPackHook(upxOptions);
            assertEquals(error, null);
            assertStringIncludes(output, expected);
        }
    });

    await t.step("validate error on non-existing upx", async () => {
        const expected = "program not found";
        let upxOptions: UpxOptions = {
            bin: "non-existing-upx",
            flags: [],
            fileList: ["dummy-file"],
            supportedPlatforms: ["windows", "linux", "darwin"],
            ignoreErrors: false,
        };
        {
            const { error } = upxPackHook(upxOptions);
            assertIsError(error as Error, Error, expected);
        }
        {
            upxOptions.ignoreErrors = true;
            const { output, error } = upxPackHook(upxOptions);
            assertEquals(error, null);
            assertStringIncludes(output, expected);
        }
    });

    await t.step("validate error on empty file list", async () => {
        const expected = "No files to pack.";
        const upxOptions: UpxOptions = {
            bin: "upx",
            flags: [],
            fileList: [],
            supportedPlatforms: ["windows", "linux", "darwin"],
            ignoreErrors: false,
        };
        {
            const { error } = upxPackHook(upxOptions);
            assertIsError(error, Error, expected);
        }
        {
            upxOptions.ignoreErrors = true;
            const { output, error } = upxPackHook(upxOptions);
            assertEquals(error, null);
            assertStringIncludes(output, expected);
        }
    });
});
