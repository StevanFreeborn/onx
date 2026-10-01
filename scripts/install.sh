#!/usr/bin/env bash
set -euo pipefail

REPO="StevanFreeborn/onx"
GITHUB_URL="https://github.com/${REPO}"
BINARY_NAME="onx"

REQUESTED_VERSION=""
INSTALL_DIR="${HOME}/.local/bin"

while [[ $# -gt 0 ]]; do
  case "$1" in
    -v|--version)
      REQUESTED_VERSION="$2"
      shift 2
      ;;
    -t|--to)
      INSTALL_DIR="$2"
      shift 2
      ;;
    -h|--help)
      cat <<EOF
onx installer script

Usage: install.sh [OPTIONS]

Options:
  -v, --version <VERSION>   Install a specific version (e.g., v0.1.0)
  -t, --to <DIR>            Install directory (default: ~/.local/bin)
  -h, --help                Show this help message
EOF
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

OS_TYPE="$(uname -s)"
case "${OS_TYPE}" in
  Linux)
    OS="unknown-linux-gnu"
    ;;
  Darwin)
    OS="apple-darwin"
    ;;
  *)
    echo "Error: Unsupported operating system: ${OS_TYPE}" >&2
    exit 1
    ;;
esac

ARCH_TYPE="$(uname -m)"
case "${ARCH_TYPE}" in
  x86_64|amd64)
    ARCH="x86_64"
    ;;
  arm64|aarch64)
    ARCH="aarch64"
    ;;
  *)
    echo "Error: Unsupported architecture: ${ARCH_TYPE}" >&2
    exit 1
    ;;
esac

TARGET="${ARCH}-${OS}"

download() {
  local url="$1"
  local output="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$output" "$url"
  else
    echo "Error: Neither curl nor wget was found. Please install one of them." >&2
    exit 1
  fi
}

if [ -z "${REQUESTED_VERSION}" ]; then
  echo "Finding latest release of ${REPO}..."

  LATEST_URL="https://api.github.com/repos/${REPO}/releases/latest"

  if command -v curl >/dev/null 2>&1; then
    RELEASE_JSON="$(curl -fsSL "${LATEST_URL}" || true)"
  elif command -v wget >/dev/null 2>&1; then
    RELEASE_JSON="$(wget -qO- "${LATEST_URL}" || true)"
  fi

  if [ -n "${RELEASE_JSON}" ]; then
    VERSION="$(echo "${RELEASE_JSON}" | grep -m 1 '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
  fi

  if [ -z "${VERSION:-}" ]; then
    echo "Could not query GitHub API for latest release, falling back to git redirect..."

    VERSION="$(curl -fsSL -o /dev/null -w "%{url_effective}" "${GITHUB_URL}/releases/latest" | rev | cut -d/ -f1 | rev)"
  fi
else
  VERSION="${REQUESTED_VERSION}"
  if [[ "${VERSION}" != v* ]]; then
    VERSION="v${VERSION}"
  fi
fi

if [ -z "${VERSION}" ]; then
  echo "Error: Could not determine version to install." >&2
  exit 1
fi

ARCHIVE_NAME="onx-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="${GITHUB_URL}/releases/download/${VERSION}/${ARCHIVE_NAME}"
CHECKSUM_URL="${DOWNLOAD_URL}.sha256"

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'onx-install')"

cleanup() {
  rm -rf "${TMP_DIR}"
}

trap cleanup EXIT

echo "Downloading ${BINARY_NAME} ${VERSION} for ${TARGET}..."

download "${DOWNLOAD_URL}" "${TMP_DIR}/${ARCHIVE_NAME}"
download "${CHECKSUM_URL}" "${TMP_DIR}/${ARCHIVE_NAME}.sha256"

echo "Verifying SHA256 checksum..."

EXPECTED_SHA="$(awk '{print $1}' "${TMP_DIR}/${ARCHIVE_NAME}.sha256")"

if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL_SHA="$(sha256sum "${TMP_DIR}/${ARCHIVE_NAME}" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL_SHA="$(shasum -a 256 "${TMP_DIR}/${ARCHIVE_NAME}" | awk '{print $1}')"
else
  echo "Warning: sha256sum/shasum not found, skipping checksum verification."
  ACTUAL_SHA="${EXPECTED_SHA}"
fi

if [ "${EXPECTED_SHA}" != "${ACTUAL_SHA}" ]; then
  echo "Error: Checksum mismatch!" >&2
  echo "  Expected: ${EXPECTED_SHA}" >&2
  echo "  Actual:   ${ACTUAL_SHA}" >&2
  exit 1
fi

echo "Extracting ${ARCHIVE_NAME}..."

mkdir -p "${TMP_DIR}/extracted"
tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "${TMP_DIR}/extracted"

EXTRACTED_BIN="$(find "${TMP_DIR}/extracted" -type f -name "${BINARY_NAME}" | head -n 1)"

if [ -z "${EXTRACTED_BIN}" ]; then
  echo "Error: Binary '${BINARY_NAME}' not found inside archive." >&2
  exit 1
fi

mkdir -p "${INSTALL_DIR}"
chmod +x "${EXTRACTED_BIN}"
mv "${EXTRACTED_BIN}" "${INSTALL_DIR}/${BINARY_NAME}"

echo "${BINARY_NAME} ${VERSION} installed successfully to ${INSTALL_DIR}/${BINARY_NAME}!"

case ":${PATH}:" in
  *:"${INSTALL_DIR}":*)
    ;;
  *)
    echo ""
    echo "Notice: ${INSTALL_DIR} is not in your PATH."

    PROFILE_FILE=""
    if [ -n "${ZSH_VERSION:-}" ] || [ -f "${HOME}/.zshrc" ]; then
      PROFILE_FILE="${HOME}/.zshrc"
    elif [ -n "${BASH_VERSION:-}" ] || [ -f "${HOME}/.bashrc" ]; then
      PROFILE_FILE="${HOME}/.bashrc"
    elif [ -f "${HOME}/.profile" ]; then
      PROFILE_FILE="${HOME}/.profile"
    fi

    if [ -n "${PROFILE_FILE}" ]; then
      PATH_EXPORT="export PATH=\"${INSTALL_DIR}:\$PATH\""
      if ! grep -qs "${INSTALL_DIR}" "${PROFILE_FILE}"; then
        echo "${PATH_EXPORT}" >> "${PROFILE_FILE}"
        echo "Added ${INSTALL_DIR} to PATH in ${PROFILE_FILE}."
        echo "Please restart your shell or run: source ${PROFILE_FILE}"
      fi
    else
      echo "To add it to your PATH, add the following line to your shell configuration:"
      echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
    ;;
esac
