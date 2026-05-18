#!/usr/bin/env bash
set -euo pipefail

if command -v tree-sitter >/dev/null 2>&1; then
  tree-sitter --version
  exit 0
fi

version="$(cat .tree-sitter-cli-version)"

if [[ "${RUNNER_OS:-}" == "Windows" ]]; then
  # Compiling tree-sitter-cli from source on Windows intermittently fails in CI.
  # The npm package ships a prebuilt binary for the GitHub-hosted Windows runner.
  npm install -g "tree-sitter-cli@${version}"
else
  cargo install tree-sitter-cli --version "${version}" --locked
fi

tree-sitter --version
