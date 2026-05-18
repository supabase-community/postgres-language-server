#!/usr/bin/env bash
set -euo pipefail

add_github_path() {
  local dir="$1"

  if [[ -n "${GITHUB_PATH:-}" ]]; then
    if [[ "${RUNNER_OS:-}" == "Windows" ]] && command -v cygpath >/dev/null 2>&1; then
      cygpath -w "${dir}" >>"${GITHUB_PATH}"
    else
      echo "${dir}" >>"${GITHUB_PATH}"
    fi
  fi
}

cargo_bin_dir() {
  local cargo_home="${CARGO_HOME:-$HOME/.cargo}"
  local dir="${cargo_home}/bin"

  if [[ "${RUNNER_OS:-}" == "Windows" ]] && command -v cygpath >/dev/null 2>&1; then
    cygpath -u "${dir}"
  else
    echo "${dir}"
  fi
}

persist_tree_sitter_path() {
  local tree_sitter_path tree_sitter_dir
  tree_sitter_path="$(command -v tree-sitter)"
  tree_sitter_dir="$(dirname "${tree_sitter_path}")"

  add_github_path "${tree_sitter_dir}"
  "${tree_sitter_path}" --version
}

cargo_bin="$(cargo_bin_dir)"
export PATH="${cargo_bin}:${PATH}"
add_github_path "${cargo_bin}"

if ! command -v tree-sitter >/dev/null 2>&1; then
  cargo install tree-sitter-cli --version "$(cat .tree-sitter-cli-version)" --locked
fi

persist_tree_sitter_path
