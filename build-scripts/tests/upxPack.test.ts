import { describe, expect, test } from "bun:test";
import type { UpxOptions } from "../bin/upxPack";
import { upxPack } from "../bin/upxPack";

describe("UPX Pack", async () => {
    test.each([
        [
            // unsupported platform
            {
                bin: "upx",
                flags: [],
                fileList: [],
                supportedPlatforms: [],
                ignoreErrors: false
            } as UpxOptions,
            /`UPX pack` is not supported or is disabled on/i
        ],
        [
            // non-existing file
            {
                bin: "upx",
                flags: [],
                fileList: ["non-existing-file"],
                supportedPlatforms: ["win32", "linux", "darwin"],
                ignoreErrors: false
            } as UpxOptions,
            /`UPX pack` failed for file `non-existing-file`/i
        ],
        [
            // non-existing upx
            {
                bin: "non-existing-upx",
                flags: [],
                fileList: ["dummy-file"],
                supportedPlatforms: ["win32", "linux", "darwin"],
                ignoreErrors: false
            } as UpxOptions,
            /`UPX pack` failed/i
        ],
        [
            // empty file list
            {
                bin: "upx",
                flags: [],
                fileList: [],
                supportedPlatforms: ["win32", "linux", "darwin"],
                ignoreErrors: false
            } as UpxOptions,
            /No files to pack/i
        ]
    ])("validate output with invalid UpxOptions", (upxOptions, expectedErrMsg) => {
        expect((): void => {
            const { error } = upxPack(upxOptions);
            throw error;
        }).toThrow(expectedErrMsg);

        {
            upxOptions.ignoreErrors = true;
            const { output, error } = upxPack(upxOptions);
            expect(error).toBeNull();
            expect(output).toMatch(expectedErrMsg);
        }
    });
});
