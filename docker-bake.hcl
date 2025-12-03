variable "CARGO_SWEEP_DAYS" { default = "7" }
variable "NO_BUNDLE" { default = "" }
variable "RUST_TARGET" { default = "x86_64-unknown-linux-gnu" }
variable "RUST_TOOLCHAIN" { default = "stable" }
variable "RUSTFLAGS" { default = "" }
# Optional secrets.
variable "GITHUB_TOKEN" { default = "" }
variable "TAURI_SIGNING_PRIVATE_KEY" { default = "" }
variable "TAURI_SIGNING_PRIVATE_KEY_PASSWORD" { default = "" }

target "base-config" {
  context    = "."
  dockerfile = "Dockerfile"
  args = {
    RUST_TOOLCHAIN   = "${RUST_TOOLCHAIN}"
    RUSTFLAGS        = "${RUSTFLAGS}"
    CARGO_SWEEP_DAYS = "${CARGO_SWEEP_DAYS}"
  }
  secret = [
    { "env":"GITHUB_TOKEN", "id": "GITHUB_TOKEN", "type": "env" }
  ]
}

target "lint" {
  inherits = ["base-config"]
  target   = "lint"
  output   = ["type=cacheonly"]
}

target "test" {
  inherits = ["base-config"]
  target   = "test"
  output   = ["type=cacheonly"]
}

target "build" {
  inherits = ["base-config"]
  target   = "artifacts"
  args = {
    RUST_TARGET = "${RUST_TARGET}"
    NO_BUNDLE    = "${NO_BUNDLE}"
  }
  secret = [
    { "env": "TAURI_SIGNING_PRIVATE_KEY", "id": "TAURI_SIGNING_PRIVATE_KEY", "type": "env" },
    { "env": "TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "id": "TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "type": "env" }
  ]
  output = ["type=local,dest=./build"]
}

target "linux" {
  inherits = ["build"]
}

target "windows" {
  inherits = ["build"]
  args = {
    RUST_TARGET = "x86_64-pc-windows-msvc"
  }
}

target "linux-no-bundle" {
  inherits = ["build"]
  args = {
    NO_BUNDLE = "1"
  }
}

target "windows-no-bundle" {
  inherits = ["build"]
  args = {
    RUST_TARGET = "x86_64-pc-windows-msvc",
    NO_BUNDLE = "1"
  }
}

# Buildx runs grouped targets in parallel, causing issues with Rust builds.
group "default" {
  targets = ["lint"]
}
