import { describe, test } from "bun:test";
import * as assert from "node:assert";
import type { UpxOptions } from "./upxPackHook";
import { upxPackHook } from "./upxPackHook";

describe("upxPackHook", async () => {
    test("validate error on unsupported platform", () => {
        const expected = /`UPX pack` is not supported on/i;
        const upxOptions: UpxOptions = {
            bin: "upx",
            flags: [],
            fileList: [],
            supportedPlatforms: [],
            ignoreErrors: false
        };
        assert.throws(
            (): void => {
                const { error } = upxPackHook(upxOptions);

                throw error;
            },
            {
                name: "Error",
                message: expected
            }
        );
        {
            upxOptions.ignoreErrors = true;
            const { output, error } = upxPackHook(upxOptions);
            assert.equal(error, null);
            assert.match(output, expected);
        }
    });

    test("validate error on non-existing file", () => {
        const expected = /`UPX pack` failed for file `non-existing-file`./i;
        const upxOptions: UpxOptions = {
            bin: "upx",
            flags: [],
            fileList: ["non-existing-file"],
            supportedPlatforms: ["win32", "linux", "darwin"],
            ignoreErrors: false
        };
        assert.throws(
            (): void => {
                const { error } = upxPackHook(upxOptions);
                throw error;
            },
            {
                name: "Error",
                message: expected
            }
        );
        {
            upxOptions.ignoreErrors = true;
            const { output, error } = upxPackHook(upxOptions);
            assert.equal(error, null);
            assert.match(output, expected);
        }
    });

    test("validate error on non-existing upx", () => {
        const expected = /`UPX pack` failed/i;
        const upxOptions: UpxOptions = {
            bin: "non-existing-upx",
            flags: [],
            fileList: ["dummy-file"],
            supportedPlatforms: ["win32", "linux", "darwin"],
            ignoreErrors: false
        };
        assert.throws(
            (): void => {
                const { error } = upxPackHook(upxOptions);
                throw error;
            },
            {
                name: "Error",
                message: expected
            }
        );
        {
            upxOptions.ignoreErrors = true;
            const { output, error } = upxPackHook(upxOptions);
            assert.equal(error, null);
            assert.match(output, expected);
        }
    });

    test("validate error on empty file list", () => {
        const expected = /No files to pack/i;
        const upxOptions: UpxOptions = {
            bin: "upx",
            flags: [],
            fileList: [],
            supportedPlatforms: ["win32", "linux", "darwin"],
            ignoreErrors: false
        };
        assert.throws(
            (): void => {
                const { error } = upxPackHook(upxOptions);
                throw error;
            },
            {
                name: "Error",
                message: expected
            }
        );
        {
            upxOptions.ignoreErrors = true;
            const { output, error } = upxPackHook(upxOptions);
            assert.equal(error, null);
            assert.match(output, expected);
        }
    });
});
