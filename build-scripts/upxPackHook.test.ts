import { assertEquals, assertIsError, assertStringIncludes } from "./deps.ts";
import { upxPackHook } from "./upxPackHook.ts";
import type { UpxOptions } from "./upxPackHook.d.ts";

Deno.test("upxPackHook", async (t) => {
    await t.step("validate error on unsupported platform", () => {
        const expected = "`UPX pack` is not supported on";
        const upxOptions: UpxOptions = {
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

    await t.step("validate error on non-existing file", () => {
        const expected = "`UPX pack` failed for file `non-existing-file`.";
        const upxOptions: UpxOptions = {
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

    await t.step("validate error on non-existing upx", () => {
        const expected = "`UPX pack` failed";
        const upxOptions: UpxOptions = {
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

    await t.step("validate error on empty file list", () => {
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
