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

persist_tree_sitter_path() {
  local tree_sitter_path tree_sitter_dir
  tree_sitter_path="$(command -v tree-sitter)"
  tree_sitter_dir="$(dirname "${tree_sitter_path}")"

  add_github_path "${tree_sitter_dir}"
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

  # npm also creates shell shims in the global bin directory. Rust's
  # Command::new("tree-sitter") needs the real binary on Windows, so expose the
  # package directory that contains tree-sitter.exe/tree-sitter directly.
  npm_tree_sitter_dir="$(npm root -g)/tree-sitter-cli"
  if [[ -d "${npm_tree_sitter_dir}" ]]; then
    export PATH="${npm_tree_sitter_dir}:${PATH}"
    add_github_path "${npm_tree_sitter_dir}"
  fi
else
  cargo install tree-sitter-cli --version "${version}" --locked
fi

persist_tree_sitter_path
