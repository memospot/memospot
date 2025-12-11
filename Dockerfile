# syntax=docker/dockerfile:1.7-labs

# Usage:
#   docker buildx bake lint
#   docker buildx bake test
#
# Managing the caches:
#   sudo docker builder du && sudo docker builder prune

ARG RUST_TOOLCHAIN=stable
ARG CARGO_SWEEP_DAYS=7
ARG RUSTFLAGS
ARG NO_BUNDLE

#? Ubuntu Version	| Ubuntu Codename	| /etc/debian_version	| Debian Version
#  24.04 LTS	    | Noble Numbat	  | trixie/sid	        | 13
#  22.04 LTS	    | Jammy Jellyfish |	bookworm/sid	      | 12
#
#* - Tauri v2 requires libwebkit2gtk-4.1-dev>=2.38, only available starting from Ubuntu 22.04/Debian 12 (bookworm).
#* - Ubuntu 22.04 repos only provide an older version of libwebkit2gtk or 2.50.1, and the latter causes AppImages to crash.
#* - Building on Debian-based images results in broken AppImage builds.
#*   Other bundles will also crash if the target machine is using this runtime version.

# Prepare base image with all necessary system packages for a Tauri build.
FROM ubuntu:24.04 AS tauri-base
ENV PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig/:/usr/share/pkgconfig"
ENV DEBIAN_FRONTEND=noninteractive
ENV HOME=/root
SHELL ["/bin/bash", "-euxo", "pipefail", "-c"]
RUN \
--mount=type=cache,target=/var/cache/apt \
<<"BASH"

  mkdir -p /etc/initramfs-tools/
  printf 'update_initramfs=no\nbackup_initramfs=no\n' > /etc/initramfs-tools/update-initramfs.conf
  rm -f /var/lib/man-db/auto-update

  apt-get update -qq

  apt-get install --no-install-recommends -qq \
    wget \
    aria2 \
    ca-certificates \
  -y
  wget -nv \
    "https://raw.githubusercontent.com/ilikenwf/apt-fast/cc0289cc45168da900939a3eabba77ae0aee25af/apt-fast" \
    -O /usr/local/bin/apt-fast
  chmod +x /usr/local/bin/apt-fast

  apt-fast install --no-install-recommends -qq \
    build-essential \
    curl \
    file \
    patchelf \
    xdg-utils \
    git \
    nsis \
    clang \
    lld \
    llvm \
    libayatana-appindicator3-dev \
    libgtk-3-dev \
    librsvg2-dev \
    libssl-dev \
    libxdo-dev \
    libwebkit2gtk-4.1-0=2.44.0-2 \
    libjavascriptcoregtk-4.1-0=2.44.0-2 \
    libwebkit2gtk-4.1-dev=2.44.0-2 \
    libjavascriptcoregtk-4.1-dev=2.44.0-2 \
    gir1.2-webkit2-4.1=2.44.0-2 \
    gir1.2-javascriptcoregtk-4.1=2.44.0-2 \
    -y

  apt-get autoremove --yes
  apt-get clean
  rm -rf /var/lib/apt/lists/*

BASH

# Toolchain image with Rust, Bun, and Tauri dependencies.
FROM tauri-base AS toolchain
ARG RUST_TOOLCHAIN
ENV PATH="/home/linuxbrew/.linuxbrew/bin:/root/.cargo/bin:/usr/local/bin:${PATH}"
SHELL ["/bin/bash", "-euxo", "pipefail", "-c"]
RUN \
--mount=type=cache,target=/root/.cargo/git \
--mount=type=cache,target=/root/.cargo/registry \
--mount=type=secret,id=GITHUB_TOKEN \
<<"BASH"

  touch /.dockerenv
  curl -fsSL -sSf https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh | bash
  eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
  brew analytics off

  brew install \
    cargo-binstall \
    rustup-init \
    oven-sh/bun/bun \
    upx \
    --force
  brew cleanup

  rustup-init -y

  binstall_cmd=(
    cargo binstall
      --disable-telemetry
      --locked
      --targets="$(rustc -vV | sed -n 's|host: ||p')"
      cargo-edit@0.13.8
      cargo-sweep@0.8.0
      cargo-xwin@0.20.2
      dprint@0.50.2
      just@1.43.0
      tauri-cli@2.9.5
      -y
  )
  if [ -s /run/secrets/GITHUB_TOKEN ]; then
    echo -e "\033[0;32mUsing GitHub token for cargo-binstall.\033[0m"
    mapfile -n1 -t token < /run/secrets/GITHUB_TOKEN
    env GITHUB_TOKEN="${token}" "${binstall_cmd[@]}"
  else
    "${binstall_cmd[@]}"
  fi

  cargo sweep -ri

BASH

FROM toolchain AS source
WORKDIR /builder
ENV CARGO_HOME="/root/.cargo"
ENV RUSTUP_HOME="/root/.rustup"
ARG RUSTFLAGS
ENV RUSTFLAGS=${RUSTFLAGS}
COPY . .

FROM source AS lint
ARG CARGO_SWEEP_DAYS
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
RUN \
--mount=type=cache,target=/root/.bun/install \
--mount=type=cache,target=/root/.cargo/git \
--mount=type=cache,target=/root/.cargo/registry \
--mount=type=cache,target=/builder/.dprint \
--mount=type=cache,target=/builder/node_modules \
--mount=type=cache,target=/builder/target \
<<BASH

  just lint
  cargo sweep -r -t ${CARGO_SWEEP_DAYS}
  cargo sweep -ri

BASH

FROM source AS test
ARG CARGO_SWEEP_DAYS
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
RUN \
--mount=type=cache,target=/root/.bun/install \
--mount=type=cache,target=/root/.cargo/git \
--mount=type=cache,target=/root/.cargo/registry \
--mount=type=cache,target=/builder/node_modules \
--mount=type=cache,target=/builder/target \
<<BASH

  just test
  cargo sweep -r -t ${CARGO_SWEEP_DAYS}
  cargo sweep -r -i

BASH

FROM source AS build
ARG CARGO_SWEEP_DAYS
ARG RUST_TARGET="x86_64-unknown-linux-gnu"
ARG NO_BUNDLE
ENV RUST_TARGET=${RUST_TARGET}
ENV XWIN_CACHE_DIR="/tmp/xwin-cache"
SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
RUN \
--mount=type=secret,id=TAURI_SIGNING_PRIVATE_KEY \
--mount=type=secret,id=TAURI_SIGNING_PRIVATE_KEY_PASSWORD \
--mount=type=cache,target=/root/.bun/install/cache \
--mount=type=cache,target=/root/.cargo/git \
--mount=type=cache,target=/root/.cargo/registry \
--mount=type=cache,target=/tmp/xwin-cache \
--mount=type=cache,target=/builder/server-dist \
--mount=type=cache,target=/builder/target \
<<"BASH"

  rustup target add ${RUST_TARGET}
  runner="cargo"
  exe=""
  if [[ "${RUST_TARGET}" == "x86_64-pc-windows-msvc" ]]; then
    runner="cargo-xwin"
    exe=".exe"
    mkdir -p "${XWIN_CACHE_DIR}"
    echo -e "\033[0;33mUsing cargo-xwin. Build may look stuck. Please wait…\033[0m"
  fi

  cmd=(cargo tauri build --runner "${runner}" --target "${RUST_TARGET}")

  if [[ -n "${NO_BUNDLE:-}" ]]; then
    cmd+=(--no-bundle)
    "${cmd[@]}"
  else
    if [ -s /run/secrets/TAURI_SIGNING_PRIVATE_KEY ] && [ -s /run/secrets/TAURI_SIGNING_PRIVATE_KEY_PASSWORD ]; then
      echo -e "\033[0;32mSigning secrets found. Bundles will be signed.\033[0m"
      mapfile -n1 -t signing_key < /run/secrets/TAURI_SIGNING_PRIVATE_KEY
      mapfile -n1 -t signing_password < /run/secrets/TAURI_SIGNING_PRIVATE_KEY_PASSWORD
      env TAURI_SIGNING_PRIVATE_KEY="${signing_key}" \
        TAURI_SIGNING_PRIVATE_KEY_PASSWORD="${signing_password}" \
        "${cmd[@]}"
    else
      echo -e "\033[0;33mSigning secrets not found. Bundles will not be signed.\033[0m"
      "${cmd[@]}" -c '{"bundle": {"createUpdaterArtifacts": false }, "plugins": {"updater": {}}}'
    fi
  fi

  cargo sweep -r -t ${CARGO_SWEEP_DAYS}
  cargo sweep -ri

  set +e
  mkdir -p /builder/build

  find "./target/${RUST_TARGET}/release/" -maxdepth 1 -type f -size +100k -exec mv {} /builder/build/ \;
  if [[ -d "./target/${RUST_TARGET}/release/bundle" ]]; then
    for dir in ./target/${RUST_TARGET}/release/bundle/*; do
      if [ -d "$dir" ]; then
        find "$dir" -maxdepth 1 -type f -exec mv {} /builder/build/ \;
      fi
    done
  fi

BASH

FROM scratch AS artifacts
WORKDIR /
COPY --from=build /builder/build/. .
