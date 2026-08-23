/// <reference types="node" />

declare module "@xhmikosr/decompress" {
    function decompress(
        input: string | Buffer,
        output?: string | decompress.DecompressOptions,
        opts?: decompress.DecompressOptions
    ): Promise<decompress.File[]>;

    namespace decompress {
        interface File {
            data: Buffer;
            mode: number;
            mtime: string;
            path: string;
            type: string;
        }

        interface DecompressOptions {
            filter?(file: File): boolean;
            map?(file: File): File;
            plugins?: any[] | undefined;
            strip?: number | undefined;
        }
    }

    export default decompress;
}
