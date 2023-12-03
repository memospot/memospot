export interface UpxOptions {
    bin: string;
    flags: string[];
    fileList: string[];
    supportedPlatforms?: string[];
    ignoreErrors?: boolean;
}
