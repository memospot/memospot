# https://earthly.dev/get-earthly
# https://docs.earthly.dev/docs/earthfile
#
# $ earthly +lint
# $ earthly +test
# $ earthly +build --target=x86_64-pc-windows-msvc
# $ earthly +build --target=x86_64-unknown-linux-gnu
#
#? Ubuntu Version	| Ubuntu Codename	| /etc/debian_version	| Debian Version
#  24.04 LTS	    | Noble Numbat	  | trixie/sid	        | 13
#  22.04 LTS	    | Jammy Jellyfish |	bookworm/sid	      | 12
#* Tauri v2 requires libwebkit2gtk-4.1-dev, which is only available starting from Ubuntu 22.04/Debian 12 (bookworm).
#* Building on Debian-based images results in broken AppImage builds.

VERSION 0.8
IMPORT github.com/earthly/lib/rust:3.0.3 AS rust
# See https://github.com/earthly/lib/tree/main/rust

ARG --global BASE_IMAGE="ubuntu:22.04"
ARG --global RUST_TOOLCHAIN="stable"
ARG --global BUN_VERSION="1.3.2"
ARG --global UPX_VERSION="5.0.2"
ARG --global RUSTFLAGS

FROM $BASE_IMAGE
WORKDIR /builder

RUN echo "\033[0;31mRUST_TOOLCHAIN=${RUST_TOOLCHAIN}\033[0m"
RUN echo "\033[0;36mRUSTFLAGS=${RUSTFLAGS}\033[0m"

BINSTALL:
  FUNCTION
  RUN cargo binstall \
    --disable-telemetry \
    --locked \
    --targets=$(rustc -vV | sed -n 's|host: ||p') \
    cargo-sweep@0.8.0 \
    cargo-edit@0.13.8 \
    cargo-xwin@0.20.2 \
    dprint@0.50.2 \
    just@1.43.0 \
    tauri-cli@2.9.4 \
    -y

SETUP_BASE_IMAGE:
  FUNCTION
  FROM $BASE_IMAGE
  ENV PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig/:/usr/share/pkgconfig"
  ENV DEBIAN_FRONTEND=noninteractive
  ENV HOME="${HOME:-/root}"

  RUN mkdir -p /etc/initramfs-tools/ && \
      echo -e 'update_initramfs=no\nbackup_initramfs=no' | tee /etc/initramfs-tools/update-initramfs.conf && \
      rm -f /var/lib/man-db/auto-update

  RUN apt-get update -qq && apt-get install wget aria2 -qqy
  RUN wget "https://raw.githubusercontent.com/ilikenwf/apt-fast/cc0289cc45168da900939a3eabba77ae0aee25af/apt-fast" -O /usr/local/bin/apt-fast && chmod +x /usr/local/bin/apt-fast

  RUN apt-fast install --no-install-recommends -qq \
    autoconf \
    autotools-dev \
    build-essential \
    ca-certificates \
    curl \
    file \
    patchelf \
    wget \
    git \
    unzip \
    nsis \
    clang \
    lld \
    llvm \
    libayatana-appindicator3-dev \
    libgtk-3-dev \
    librsvg2-dev \
    libssl-dev \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    jq \
    -y

SETUP_RUST:
  FUNCTION
  ENV PATH="${HOME}/.cargo/bin:${PATH}"
  RUN curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | bash -s -- --profile default --default-toolchain $RUST_TOOLCHAIN -y
  RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

SETUP_BUN:
  FUNCTION
  ENV BUN_INSTALL="${HOME}/.bun"
  ENV PATH="${BUN_INSTALL}/bin:${PATH}"
  RUN curl -fsSL https://bun.sh/install | bash -s "bun-v${BUN_VERSION}"
  CACHE ${BUN_INSTALL}/install/cache/

SETUP_UPX:
  FUNCTION
  RUN wget -nv -O- "https://github.com/upx/upx/releases/download/v${UPX_VERSION}/upx-${UPX_VERSION}-amd64_linux.tar.xz" | tar -xJ upx-${UPX_VERSION}-amd64_linux/upx && mv upx-${UPX_VERSION}-amd64_linux/upx /usr/local/bin/upx

CARGOSWEEP:
  FUNCTION
  DO rust+CHECK_INITED
  ARG output
  DO rust+SET_CACHE_MOUNTS_ENV
  RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE --mount=$EARTHLY_RUST_TARGET_CACHE \
    set -e; \
    cargo sweep -r -t $EARTHLY_SWEEP_DAYS; \
    cargo sweep -r -i; \
    $EARTHLY_FUNCTIONS_HOME/copy-output.sh "$output";
  RUN $EARTHLY_FUNCTIONS_HOME/rename-output.sh

install:
  DO +SETUP_BASE_IMAGE
  DO +SETUP_BUN
  DO +SETUP_UPX
  DO +SETUP_RUST
  DO +BINSTALL

  # Call +INIT before copying the source files to avoid installing depencies every time source code changes.
  # This parametrization will be used in future calls to functions of the library.
  DO rust+INIT --keep_fingerprints=true

source:
  FROM +install
  WORKDIR /builder
  COPY --keep-ts . .

lint:
  FROM +source
  CACHE /builder/.dprint/
  DO rust+SET_CACHE_MOUNTS_ENV
  RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE --mount=$EARTHLY_RUST_TARGET_CACHE \
    just lint
  DO +CARGOSWEEP

test:
  FROM +source
  DO rust+SET_CACHE_MOUNTS_ENV
  RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE --mount=$EARTHLY_RUST_TARGET_CACHE \
    just test
  DO +CARGOSWEEP

build:
  FROM +source
  ARG nosign
  ARG nobundle
  ARG target="x86_64-unknown-linux-gnu"

  LET exe=""
  LET runner="cargo"
  IF [ ${target} = "x86_64-pc-windows-msvc" ]
    SET exe=".exe"
    SET runner="cargo-xwin"
    ENV XWIN_CACHE_DIR="/tmp/xwin-cache"
    CACHE $XWIN_CACHE_DIR
    RUN echo "\033[0;31mUsing cargo-xwin. Build may look stuck.\033[0m"
  END

  DO rust+SET_CACHE_MOUNTS_ENV
  RUN rustup target add ${target}
  CACHE /builder/server-dist/

  IF [ -z "$nosign" ] && [ -z "$nobundle" ]
    RUN echo "\033[0;31mSigning is enabled. Testing if secret keys are set.\033[0m"
    RUN echo "\033[0;35mYou may pass --nosign=1 to disable signing.\033[0m"
    RUN --secret TAURI_SIGNING_PRIVATE_KEY --secret TAURI_SIGNING_PRIVATE_KEY_PASSWORD \
        echo "\033[0;32mSecret keys are set.\033[0m"
  END

  IF [ -n "$nobundle" ]
    RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE --mount=$EARTHLY_RUST_TARGET_CACHE \
      cargo tauri build --runner ${runner} --target ${target} --no-bundle
  ELSE IF [ -n "$nosign" ]
    RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE --mount=$EARTHLY_RUST_TARGET_CACHE \
      cargo tauri build --runner ${runner} --target ${target} -c '{"bundle": {"createUpdaterArtifacts": false }, "plugins": {"updater": {}}}'
  ELSE
    RUN --mount=$EARTHLY_RUST_CARGO_HOME_CACHE --mount=$EARTHLY_RUST_TARGET_CACHE \
        --secret TAURI_SIGNING_PRIVATE_KEY --secret TAURI_SIGNING_PRIVATE_KEY_PASSWORD \
          cargo tauri build --runner ${runner} --target ${target}
  END
  DO +CARGOSWEEP

  RUN --mount=$EARTHLY_RUST_TARGET_CACHE \
    set +e; \
    mkdir ./build/; \
    mv ./target/${target}/release/memospot${exe} ./build/; \
    mv ./target/${target}/release/memos${exe} ./build/; \
    mv ./target/${target}/release/bundle/ ./build/; \
    exit 0
  SAVE ARTIFACT --keep-ts --if-exists ./build/bundle/nsis/* AS LOCAL ./build/
  SAVE ARTIFACT --keep-ts --if-exists ./build/bundle/deb/*.deb AS LOCAL ./build/
  SAVE ARTIFACT --keep-ts --if-exists ./build/bundle/rpm/*.rpm AS LOCAL ./build/
  SAVE ARTIFACT --keep-ts --if-exists ./build/bundle/appimage/*.AppImage* AS LOCAL ./build/
  SAVE ARTIFACT --keep-ts ./build/memospot${exe} AS LOCAL ./build/
  SAVE ARTIFACT --keep-ts ./build/memos${exe} AS LOCAL ./build/
