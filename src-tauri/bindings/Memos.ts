// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

/**
 * Memos configuration.
 */
export type Memos = { 
/**
 * Memos binary path.
 */
binary_path: string | null, 
/**
 * Memos current working directory.
 */
working_dir: string | null, 
/**
 * Directory where Memos will store its database and assets.
 */
data: string | null, 
/**
 * Server mode. Each mode uses a different database file.
 *
 * Can be one of:
 * - prod
 * - dev
 * - demo
 */
mode: string | null, 
/**
 * Server address.
 *
 * This should be "127.0.0.1" whenever running under Memospot.
 *
 * Binding to all addresses "0.0.0.0" will trigger a firewall warning on Windows.
 */
addr: string | null, 
/**
 * Last port used by Memos.
 *
 * Memospot will try to reuse this port on subsequent runs, and will find a new
 * free port if the previous one is already in use or if this value is set to 0.
 */
port: number | null, 
/**
 * Custom environment variables to pass to Memos.
 */
env: { [key in string]?: string } | null, };
