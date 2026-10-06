#!/usr/bin/env bash
set -euo pipefail

# Termarx Linux Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/icodejoo/termarx/main/scripts/install-linux.sh | bash

REPO="icodejoo/termarx"
INSTALL_DIR="${TERMY_INSTALL_DIR:-$HOME/.local/bin}"

die() {
  echo "Error: $*" >&2
  exit 1
}

log() {
  echo "==> $*"
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "'$1' is required but not found"
}

json_string_values() {
  local key="$1"
  { grep -Eo "\"${key}\"[[:space:]]*:[[:space:]]*\"[^\"]*\"" || true; } \
    | sed -n 's/.*"[[:space:]]*:[[:space:]]*"\([^"]*\)"$/\1/p'
}

detect_arch() {
  local arch
  arch="$(uname -m)"
  case "$arch" in
    x86_64|amd64) echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *) die "Unsupported architecture: $arch" ;;
  esac
}

require_cmd curl
require_cmd tar
require_cmd grep
require_cmd sed

log "Detecting system architecture..."
ARCH="$(detect_arch)"
log "Architecture: $ARCH"

log "Fetching latest release from GitHub..."
RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest")"

TAG="$(printf '%s\n' "$RELEASE_JSON" | json_string_values "tag_name" | head -n1)"
if [[ -z "$TAG" ]]; then
  die "Could not determine latest release tag"
fi
log "Latest version: $TAG"

DOWNLOAD_URL="$(
  printf '%s\n' "$RELEASE_JSON" \
    | json_string_values "browser_download_url" \
    | grep -E "linux.*${ARCH}.*\.tar\.gz$" \
    | head -n1 \
    || true
)"

if [[ -z "$DOWNLOAD_URL" ]]; then
  DOWNLOAD_URL="$(
    printf '%s\n' "$RELEASE_JSON" \
      | json_string_values "browser_download_url" \
      | grep -E "linux.*\.tar\.gz$" \
      | grep -Ev 'linux.*(x86_64|amd64|aarch64|arm64).*\.tar\.gz$' \
      | head -n1 \
      || true
  )"
fi

if [[ -z "$DOWNLOAD_URL" ]]; then
  die "Could not find Linux tarball for architecture '$ARCH' in release $TAG"
fi

log "Download URL: $DOWNLOAD_URL"

TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

TARBALL_PATH="$TEMP_DIR/termarx.tar.gz"

log "Downloading Termarx $TAG..."
curl -fsSL "$DOWNLOAD_URL" -o "$TARBALL_PATH"

log "Extracting..."
tar -xzf "$TARBALL_PATH" -C "$TEMP_DIR"

BINARY_PATH=""
if [[ -f "$TEMP_DIR/termarx/termarx-bin" ]]; then
  BINARY_PATH="$TEMP_DIR/termarx/termarx-bin"
elif [[ -f "$TEMP_DIR/termarx/termarx" ]]; then
  BINARY_PATH="$TEMP_DIR/termarx/termarx"
elif [[ -f "$TEMP_DIR/termarx-bin" ]]; then
  BINARY_PATH="$TEMP_DIR/termarx-bin"
elif [[ -f "$TEMP_DIR/termarx" ]]; then
  BINARY_PATH="$TEMP_DIR/termarx"
else
  BINARY_PATH="$(find "$TEMP_DIR" \( -name "termarx-bin" -o -name "termarx" \) -type f -executable 2>/dev/null | head -n1)"
fi

if [[ -z "$BINARY_PATH" || ! -f "$BINARY_PATH" ]]; then
  die "Could not find termarx binary in downloaded tarball"
fi

CLI_BINARY_PATH=""
if [[ -f "$TEMP_DIR/termarx/termarx-cli" ]]; then
  CLI_BINARY_PATH="$TEMP_DIR/termarx/termarx-cli"
else
  CLI_BINARY_PATH="$(find "$TEMP_DIR" -name "termarx-cli" -type f -executable 2>/dev/null | head -n1)"
fi

if [[ -z "$CLI_BINARY_PATH" || ! -f "$CLI_BINARY_PATH" ]]; then
  die "Could not find termarx-cli binary in downloaded tarball"
fi

mkdir -p "$INSTALL_DIR"

log "Installing to $INSTALL_DIR/termarx..."
rm -f "$INSTALL_DIR/termarx" "$INSTALL_DIR/termarx-bin" "$INSTALL_DIR/termarx-cli"
cp "$BINARY_PATH" "$INSTALL_DIR/termarx-bin"
cp "$CLI_BINARY_PATH" "$INSTALL_DIR/termarx-cli"
cat > "$INSTALL_DIR/termarx" <<'LAUNCHER'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${TERMY_LINUX_BACKEND:-x11}" == "x11" && -n "${DISPLAY:-}" ]]; then
  unset WAYLAND_DISPLAY
fi

exec "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/termarx-bin" "$@"
LAUNCHER
chmod +x "$INSTALL_DIR/termarx" "$INSTALL_DIR/termarx-bin" "$INSTALL_DIR/termarx-cli"

if [[ -d "$TEMP_DIR/termarx/file-manager" ]]; then
  SHARE_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
  mkdir -p \
    "$SHARE_HOME/applications" \
    "$SHARE_HOME/kio/servicemenus" \
    "$SHARE_HOME/kservices5/ServiceMenus" \
    "$SHARE_HOME/nemo/actions" \
    "$SHARE_HOME/nautilus/scripts" \
    "$SHARE_HOME/caja/scripts"
  if [[ -f "$TEMP_DIR/termarx/file-manager/termarx.desktop" ]]; then
    cp "$TEMP_DIR/termarx/file-manager/termarx.desktop" "$SHARE_HOME/applications/termarx.desktop"
  fi
  if [[ -f "$TEMP_DIR/termarx/file-manager/termarx-open-tab.desktop" ]]; then
    cp "$TEMP_DIR/termarx/file-manager/termarx-open-tab.desktop" "$SHARE_HOME/kio/servicemenus/termarx-open-tab.desktop"
    cp "$TEMP_DIR/termarx/file-manager/termarx-open-tab.desktop" "$SHARE_HOME/kservices5/ServiceMenus/termarx-open-tab.desktop"
  fi
  if [[ -f "$TEMP_DIR/termarx/file-manager/termarx-open-tab.nemo_action" ]]; then
    cp "$TEMP_DIR/termarx/file-manager/termarx-open-tab.nemo_action" "$SHARE_HOME/nemo/actions/termarx-open-tab.nemo_action"
  fi
  if [[ -f "$TEMP_DIR/termarx/file-manager/nautilus-open-tab.sh" ]]; then
    install -m 755 "$TEMP_DIR/termarx/file-manager/nautilus-open-tab.sh" "$SHARE_HOME/nautilus/scripts/Open new Termarx tab here"
    install -m 755 "$TEMP_DIR/termarx/file-manager/nautilus-open-tab.sh" "$SHARE_HOME/caja/scripts/Open new Termarx tab here"
  fi
fi

log "Termarx $TAG installed successfully!"

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo ""
  echo "NOTE: $INSTALL_DIR is not in your PATH."
  echo "Add it to your shell config:"
  echo ""
  echo "  # For bash (~/.bashrc):"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo ""
  echo "  # For zsh (~/.zshrc):"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo ""
  echo "  # For fish (~/.config/fish/config.fish):"
  echo "  set -gx PATH \$HOME/.local/bin \$PATH"
  echo ""
fi

echo ""
echo "Run 'termarx' to start the terminal."
