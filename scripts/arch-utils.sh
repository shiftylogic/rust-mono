#
# This script is not meant to be run alone, but rather sourced / included
# as part of the execution of other scripts.
#

case "$OSTYPE" in
  darwin*)
    SYS_OS="darwin"
    SYS_ARCH="$(uname -m)"
    SYS_ARCH_ALT="$SYS_ARCH"
    if [[ "$SYS_ARCH" == "arm64" ]]; then SYS_ARCH_ALT="aarch64"; fi
    ;;
  linux*)
    SYS_OS="linux"
    SYS_ARCH="$(uname -m)"
    SYS_ARCH_ALT="$SYS_ARCH"
    ;;
  msys*|cygwin*|win*)
    SYS_OS="windows"
    SYS_ARCH="x86_64"
    ;;
  *)
    echo "ERROR: Unsupported platform."
    ;;
esac

