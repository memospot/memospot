import { describe, expect, it } from "vitest";
import { envFromKV, envToKV } from "./environmentVariables";

describe("envToKV function", () => {
    const testCases = [
        {
            input: "KEY1=value1\nKEY2=value2",
            expected: { KEY1: "value1", KEY2: "value2" }
        },
        {
            input: "KEY1= value with spaces \nKEY2=  another value  ",
            expected: { KEY1: " value with spaces ", KEY2: "  another value  " }
        },
        {
            input: "KEY1=value1\r\n\nKEY2=value2\r",
            expected: { KEY1: "value1", KEY2: "value2" }
        },
        {
            input: "=value\n KEY1=value1\nINVALID\nKEY2=value2",
            expected: { KEY1: "value1", KEY2: "value2" }
        }
    ];

    testCases.forEach((testCase, index) => {
        it(`should correctly parse test case ${index + 1}`, () => {
            const result = envToKV(testCase.input);
            expect(result).toEqual(testCase.expected);
        });
    });

    it("should handle empty input", () => {
        expect(envToKV("")).toEqual({});
    });
});

describe("envFromKV function", () => {
    const testCases = [
        {
            input: { KEY1: "value1", KEY2: "value2" },
            expected: "KEY1=value1\nKEY2=value2"
        },
        {
            input: { KEY1: " value with spaces ", KEY2: "  another value  " },
            expected: "KEY1= value with spaces \nKEY2=  another value  "
        }
    ];

    testCases.forEach((testCase, index) => {
        it(`should correctly parse test case ${index + 1}`, () => {
            const result = envFromKV(testCase.input);
            expect(result).toEqual(testCase.expected);
        });
    });
});
