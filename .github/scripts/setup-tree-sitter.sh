#!/usr/bin/env bash
set -euo pipefail

persist_tree_sitter_path() {
  local tree_sitter_path tree_sitter_dir
  tree_sitter_path="$(command -v tree-sitter)"
  tree_sitter_dir="$(dirname "${tree_sitter_path}")"

  if [[ -n "${GITHUB_PATH:-}" ]]; then
    if [[ "${RUNNER_OS:-}" == "Windows" ]] && command -v cygpath >/dev/null 2>&1; then
      cygpath -w "${tree_sitter_dir}" >>"${GITHUB_PATH}"
    else
      echo "${tree_sitter_dir}" >>"${GITHUB_PATH}"
    fi
  fi

  "${tree_sitter_path}" --version
}

if command -v tree-sitter >/dev/null 2>&1; then
  persist_tree_sitter_path
  exit 0
fi

version="$(cat .tree-sitter-cli-version)"

if command -v npm >/dev/null 2>&1; then
  # The npm package ships prebuilt binaries, avoiding slow and flaky source
  # compilation on GitHub-hosted runners.
  npm install -g "tree-sitter-cli@${version}"
else
  cargo install tree-sitter-cli --version "${version}" --locked
fi

persist_tree_sitter_path
