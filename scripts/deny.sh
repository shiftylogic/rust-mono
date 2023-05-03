#!/usr/bin/env bash

set -eu

SCRIPT_ROOT=`dirname $0`
LOCAL_ROOT=$SCRIPT_ROOT/..

# Import the architecture detection
source $SCRIPT_ROOT/arch-utils.sh

# Setup the local variables
CARGO_DENY_VER="$(cat ${LOCAL_ROOT}/.cargo-deny-version)"
CARGO_DENY_REL_BASE="https://github.com/EmbarkStudios/cargo-deny/releases/download"

case "$SYS_OS" in
  darwin)
    CARGO_DENY_ARCH="$SYS_ARCH_ALT-apple-darwin"
    ;;
  linux)
    CARGO_DENY_ARCH="$SYS_ARCH-unknown-linux-musl"
    ;;
  windows)
    CARGO_DENY_ARCH="$SYS_ARCH-pc-windows-msvc"
    ;;
esac
CARGO_DENY_REL="$CARGO_DENY_REL_BASE/$CARGO_DENY_VER/cargo-deny-$CARGO_DENY_VER-$CARGO_DENY_ARCH.tar.gz"

TARGET_TOOLS_DIR="$LOCAL_ROOT/.tools"
TARGET_BIN="cargo-deny"
if [[ "$SYS_OS" == "windows" ]]; then TARGET_BIN="$TARGET_BIN.exe"; fi

# Check if we already downloaded it.
FORCE_FLAG=${1:-""}
if [[ ! -x "$TARGET_TOOLS_DIR/$TARGET_BIN" ]];
then
  echo "Downloading cargo-deny binary..."

  # Make sure the tools location already exists
  mkdir -p $TARGET_TOOLS_DIR

  # Download the cargo-deny pre-built
  curl -s -L $CARGO_DENY_REL | tar -xzf- --strip-components=1 --include="*/cargo-deny*" -C $TARGET_TOOLS_DIR
fi

# Now run the tool on the repo
$TARGET_TOOLS_DIR/$TARGET_BIN --workspace check

