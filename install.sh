#!/bin/sh

set -eu

PROJECT_NAME="tm"
REPO="${TM_REPO:-fischcheng/template-manager-cli-agent}"
INSTALL_DIR="${TM_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${TM_VERSION:-latest}"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: required command not found: $1" >&2
    exit 1
  fi
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Darwin) os_part="apple-darwin" ;;
    Linux) os_part="unknown-linux-gnu" ;;
    *)
      echo "error: unsupported operating system: $os" >&2
      exit 1
      ;;
  esac

  case "$arch" in
    arm64|aarch64) arch_part="aarch64" ;;
    x86_64|amd64) arch_part="x86_64" ;;
    *)
      echo "error: unsupported architecture: $arch" >&2
      exit 1
      ;;
  esac

  printf '%s-%s\n' "$arch_part" "$os_part"
}

build_download_url() {
  target="$1"

  if [ "$VERSION" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download/%s-%s.tar.gz\n' "$REPO" "$PROJECT_NAME" "$target"
  else
    printf 'https://github.com/%s/releases/download/%s/%s-%s.tar.gz\n' "$REPO" "$VERSION" "$PROJECT_NAME" "$target"
  fi
}

download() {
  url="$1"
  output="$2"

  if command -v curl >/dev/null 2>&1; then
    curl --fail --location --silent --show-error "$url" --output "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$output" "$url"
  else
    echo "error: install requires curl or wget" >&2
    exit 1
  fi
}

ensure_install_dir() {
  if [ ! -d "$INSTALL_DIR" ]; then
    mkdir -p "$INSTALL_DIR"
  fi
}

main() {
  need_cmd uname
  need_cmd tar
  need_cmd mktemp

  target="$(detect_target)"
  archive_url="$(build_download_url "$target")"
  tmp_dir="$(mktemp -d)"
  archive_path="$tmp_dir/$PROJECT_NAME.tar.gz"

  trap 'rm -rf "$tmp_dir"' EXIT INT TERM

  echo "Installing $PROJECT_NAME for $target from $archive_url"
  download "$archive_url" "$archive_path"

  tar -xzf "$archive_path" -C "$tmp_dir"

  if [ ! -f "$tmp_dir/$PROJECT_NAME" ]; then
    echo "error: release archive did not contain $PROJECT_NAME" >&2
    exit 1
  fi

  ensure_install_dir
  cp "$tmp_dir/$PROJECT_NAME" "$INSTALL_DIR/$PROJECT_NAME"
  chmod 755 "$INSTALL_DIR/$PROJECT_NAME"

  echo "Installed $PROJECT_NAME to $INSTALL_DIR/$PROJECT_NAME"

  case ":$PATH:" in
    *:"$INSTALL_DIR":*)
      ;;
    *)
      echo "warning: $INSTALL_DIR is not on PATH" >&2
      echo "Add this to your shell profile:" >&2
      echo "  export PATH=\"$INSTALL_DIR:\$PATH\"" >&2
      ;;
  esac

  echo "Run '$PROJECT_NAME --help' to get started."
}

main "$@"
