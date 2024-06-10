#!/usr/bin/env bash
set -euo pipefail

platform=$(uname -ms)

if [[ ${OS:-} = Windows_NT ]]; then
    if [[ $platform != MINGW64* ]]; then
        powershell -c "Set-ExecutionPolicy Bypass -Scope Process -Force; irm https://raw.githubusercontent.com/memospot/memospot/main/memos-server-updater.ps1|iex"
        exit $?
    else
        echo "Cygwin is not supported by this script."
        exit 1
    fi
fi

# Reset
Color_Off=''

# Regular Colors
Dim='' # White
Green=''
Red=''
Yellow=''

# Bold
Bold_Cyan=''
Bold_Green=''
Bold_White=''
Bold_Yellow=''

if [[ -t 1 ]]; then
    # Reset
    Color_Off='\033[0m' # Text Reset

    # Regular Colors
    Dim='\033[0;2m'
    Green='\033[0;32m'  # Green
    Red='\033[0;31m'    # Red
    Yellow='\033[0;33m' # Yellow

    # Bold
    Bold_Cyan='\033[1;36m'   # Bold Cyan
    Bold_Green='\033[1;32m'  # Bold Green
    Bold_Red='\033[1;31m'    # Bold Red
    Bold_White='\033[1m'     # Bold White
    Bold_Yellow='\033[1;33m' # Bold Yellow
fi

error() {
    echo -e "${Red}error${Color_Off}:" "$@" >&2
    exit 1
}

info() {
    echo -e "${Dim}$* ${Color_Off}"
}

msg() {
    echo -e "$* ${Color_Off}"
}

warn() {
    echo -e "${Yellow}$* ${Color_Off}"
}

info_bold() {
    echo -e "${Bold_White}$* ${Color_Off}"
}

success() {
    echo -e "${Green}$* ${Color_Off}"
}

cpu_supports_avx2() {
    case $platform in
    'Darwin x86_64')
        if sysctl -a | grep machdep.cpu | grep -q AVX2; then
            return 0
        fi
        ;;
    'Linux x86_64' | *)
        if grep -q avx2 </proc/cpuinfo; then
            return 0
        fi
        ;;
    esac
    return 1
}

cpu_supports_popcnt() {
    case $platform in
    'Darwin x86_64')
        if sysctl -a | grep machdep.cpu | grep -q POPCNT; then
            return 0
        fi
        ;;
    'Linux x86_64' | *)
        if grep -q popcnt </proc/cpuinfo; then
            return 0
        fi
        ;;
    esac
    return 1
}

################################################################################
main() {

    banner=$(
        cat <<"EOF"
         __  __ _____ __  __  ___  ____  ____   ___ _____
        |  \/  | ____|  \/  |/ _ \/ ___||  _ \ / _ \_   _|
        | |\/| |  _| | |\/| | | | \___ \| |_) | | | || |
        | |  | | |___| |  | | |_| |___) |  __/| |_| || |
        |_|  |_|_____|_|  |_|\___/|____/|_|    \___/ |_|
                                                 _       _
 ___  ___ _ ____   _____ _ __    _   _ _ __   __| | __ _| |_ ___ _ __
/ __|/ _ \ '__\ \ / / _ \ '__|  | | | | '_ \ / _` |/ _` | __/ _ \ '__|
\__ \  __/ |   \ V /  __/ |     | |_| | |_) | (_| | (_| | ||  __/ |
|___/\___|_|    \_/ \___|_|      \__,_| .__/ \__,_|\__,_|\__\___|_|
                                      |_|
EOF
    )
    echo -e "${Bold_Cyan}${banner}${Color_Off}"
    echo -e "${Bold_Green}https://memospot.github.io/ | https://github.com/memospot/memospot${Color_Off}\n"

    command -v tar >/dev/null ||
        error "tar is required to extract Memos' updates"
    command -v jq >/dev/null ||
        error "jq is required to get latest Memos' tag"

    if [[ $# -gt 1 ]]; then
        error 'Too many arguments, only one is allowed. The allowed argument can be a specific tag of Memos to install. (e.g. "v0.22.0")'
    fi

    case $platform in
    'Darwin x86_64')
        if cpu_supports_avx2; then
            target=darwin-x86_64_v3
        elif cpu_supports_popcnt; then
            target=darwin-x86_64_v2
        else
            target=darwin-x86_64
        fi
        ;;
    'Darwin arm64')
        target=darwin-arm64
        ;;
    'Linux aarch64' | 'Linux arm64')
        target=linux-arm64
        ;;
    'Linux x86_64' | *)
        if cpu_supports_avx2; then
            target=linux-x86_64_v3
        elif cpu_supports_popcnt; then
            target=linux-x86_64_v2
        else
            target=linux-x86_64
        fi
        ;;
    esac

    if [[ $target = darwin-arm64 ]]; then
        # Is this process running in Rosetta?
        # redirect stderr to devnull to avoid error message when not running in Rosetta
        if [[ $(sysctl -n sysctl.proc_translated 2>/dev/null) = 1 ]]; then
            target=darwin-arm64
            info "Your shell is running in Rosetta 2. Downloading Memos for $target instead"
        fi
    fi

    GITHUB=${GITHUB-"https://github.com"}
    REPO=${REPO-"memospot/memos-builds"}
    repo_url="$GITHUB/$REPO"
    latest_tag=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | jq -r '.tag_name') || error "Failed to fetch latest tag from GitHub."
    install_tag=${1-$latest_tag}
    download_file=memos-$install_tag-$target.tar.gz
    sha256sums_uri=$(curl -s "https://api.github.com/repos/$REPO/releases/tags/$install_tag" | jq -r '.assets[] | select(.name|endswith(".txt")) | .browser_download_url') || error "Failed to fetch tag $install_tag from GitHub."
    memos_uri=$repo_url/releases/download/$install_tag/$download_file

    INSTALL_DIR=${INSTALL_DIR-"/usr/bin"}
    SUDO_USER=${SUDO_USER-"$USER"}
    USER_HOME=$(getent passwd "$SUDO_USER" | cut -d: -f6)

    locations=(
        "$INSTALL_DIR"
        "/usr/local/bin"
        "$USER_HOME/.local/bin"
    )
    for location in "${locations[@]}"; do
        if [[ -f "$location/memos" ]]; then
            INSTALL_DIR="$location"
            memos_bin="$location/memos"
            break
        fi
    done
    if [[ -z $memos_bin ]]; then
        error "Memos' binary was not found in any of the following locations: ${Bold_Red}${locations[*]}"
    fi

    info "· Found Memos' binary at $Bold_White$memos_bin"

    if [[ ! -w $memos_bin ]]; then
        error "Memos' binary at $memos_bin is not writable. Please run this script with sudo."
    fi

    msg "· Latest Memos' release found: $Bold_White$latest_tag"
    msg "· Selected tag: $Bold_Cyan$install_tag$Dim (You may pass a different tag as an argument to this script)"
    msg "· Repository URL: $Bold_White$repo_url"
    info "· Release URL: $Bold_White$memos_uri"
    info "· Release SHA256SUMS: $Bold_White$sha256sums_uri"

    warn "\nAvoid doing Major and Minor version updates. Memospot may not be able to handle them yet."
    warn "Version scheme is Major.Minor.Patch. ${Bold_Yellow}Patch${Yellow} version updates should be fine."
    warn "It's essential to back-up your database before trying a major or minor version update."

    msg "\n» This script will not check whether you need to update the Memos server."
    msg "» Version $install_tag will be downloaded and installed regardless."

    info "$Bold_Green\n-> Press any key to continue <-\n"
    read -n 1 -sr </dev/tty

    info "Downloading Memos $install_tag ($target) from $Bold_White$memos_uri"
    (cd /tmp &&
        curl --fail --location --progress-bar --output "$download_file" "$memos_uri") ||
        error "Failed to download Memos from \"$memos_uri\"."

    info "Downloading SHA256SUMS from $Bold_White$sha256sums_uri"
    (cd /tmp &&
        curl --fail --location --progress-bar --output "memos_sha256sums.txt" "$sha256sums_uri") ||
        error "Failed to download sha256sums from \"$sha256sums_uri\"."

    info "Verifying SHA256SUMS"
    (cd /tmp && sha256sum --ignore-missing --quiet --check "memos_sha256sums.txt") ||
        (cd /tmp && shasum -a 256 --binary --status --check "memos_sha256sums.txt") ||
        warn "Failed to verify SHA256SUMS."

    tar -xzf "/tmp/$download_file" -C "$INSTALL_DIR" memos ||
        error "Failed to extract Memos from \"/tmp/$download_file\"."

    rm "/tmp/$download_file"
    rm "/tmp/memos_sha256sums.txt"

    chmod +x "$memos_bin" ||
        error 'Failed to set permissions on Memos executable'

    success "Memos $install_tag was installed successfully to $INSTALL_DIR"

    database="$USER_HOME/.memospot/memos_prod.db"
    if [[ -f "$database" ]]; then
        backup_dir="$USER_HOME/.memospot/server-updater-backups"
        date_string=$(date +%Y-%m-%d_%H-%M-%S)
        mkdir -p "$backup_dir" || error "Failed to create backup directory."
        info "Backing up database to $Bold_White$backup_dir"

        file_list=("memos_prod.db")
        if [[ -f "$database-wal" ]]; then
            file_list+=("memos_prod.db-wal")
        fi
        if [[ -f "$database-shm" ]]; then
            file_list+=("memos_prod.db-shm")
        fi

        (cd "$backup_dir/../" && tar -czf "$backup_dir/memos_prod-${date_string}_before_${install_tag}.tar.gz" "${file_list[@]}") || error "Failed to backup database."
        success "Database backup was created at $Bold_Green$backup_dir"

        chown -R "$SUDO_USER":"$SUDO_USER" "$backup_dir"
    fi

    success "Update complete."
}

main "$@" || exit 1
