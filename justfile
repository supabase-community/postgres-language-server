_default:
  just --list -u

alias f := format
alias r := ready
alias l := lint
alias t := test
alias rg := reset-git
alias qm := quick-modify

# Prepares a fresh checkout or worktree: the libpg_query submodule and the JS dependencies
setup:
	git submodule update --init --recursive
	bun install --frozen-lockfile

# Installs the tools needed to develop
install-tools:
	cargo install cargo-binstall
	cargo install tree-sitter-cli --version "$(cat .tree-sitter-cli-version)" --locked
	cargo binstall cargo-insta taplo-cli sqlx-cli
	cargo binstall --git "https://github.com/astral-sh/uv" uv
	bun install

# Upgrades the tools needed to develop
upgrade-tools:
	cargo install cargo-binstall --force
	cargo install tree-sitter-cli --version "$(cat .tree-sitter-cli-version)" --locked --force
	cargo binstall cargo-insta taplo-cli sqlx-cli --force
	cargo binstall --git "https://github.com/astral-sh/uv" uv --force

# Generates code generated files for the linter
gen-lint:
  cargo run -p xtask_codegen -- analyser
  cargo run -p xtask_codegen -- configuration
  cargo run -p xtask_codegen -- bindings
  cargo run -p xtask_codegen -- schema-types
  cargo run -p xtask_codegen -- splinter
  cargo run -p xtask_codegen -- pglinter
  cargo run -p rules_check
  cargo run -p docs_codegen
  just format

# Creates a new lint rule in the given path, with the given name. Name has to be camel case. Group should be lowercase.
new-lintrule group rulename severity="error":
  cargo run -p xtask_codegen -- new-lintrule --category=lint --name={{rulename}} --group={{group}} --severity={{severity}}
  just gen-lint

# Format Rust, JS and TOML files
format:
	cargo fmt
	taplo format
	bun oxfmt packages/

format-ci:
	cargo fmt --all --check
	taplo format --check
	bun oxfmt --check packages/

format-ci-versions:
	cargo --version
	taplo --version
	bun oxfmt --version

[unix]
_touch file:
  touch {{file}}

[windows]
_touch file:
  (gci {{file}}).LastWriteTime = Get-Date

# Records the Postgres regression fixtures of a major version (needs Docker), e.g. just record-regress 17 REL_17_12
# Pass --catalog-only to re-record only the catalog, e.g. after the snapshot queries change.
record-regress major *args:
	cargo run -p pgls_postgres_regress --features record --bin record -- {{major}} {{args}}

# Starts the test database and waits until it accepts connections. Rebuilds the image first; with the Docker build cache, that takes seconds unless the Dockerfile changed.
db:
	docker compose up -d --build --wait

# Run tests of all crates
test:
	cargo test --workspace --no-fail-fast

# Run tests for the crate passed as argument e.g. just test-crate pgls_cli
test-crate name:
	cargo test -p {{name}} --no-fail-fast

# Run doc tests
test-doc:
	cargo test --doc

# Alias for `cargo clippy`, it runs clippy on the whole codebase
lint:
  cargo clippy
  cargo run -p rules_check
  bun oxlint packages/

# Runs what CI's Rust jobs gate on: formatting, clippy with warnings as errors, and a clean codegen
check:
  just format-ci
  cargo clippy --workspace --all-targets -- -D warnings
  just check-codegen

# Compares a snapshot of the working tree (tracked and untracked files) before and after, so it also works with uncommitted changes.
# Fails if `just gen-lint` changes any file, like CI's "Check Codegen" job
check-codegen:
  #!/usr/bin/env bash
  set -euo pipefail
  # Writes the working tree to a tree object through a throwaway copy of the index.
  snapshot() {
    local index
    index=$(mktemp)
    cp "$(git rev-parse --git-path index)" "$index"
    GIT_INDEX_FILE="$index" git add -A
    GIT_INDEX_FILE="$index" git write-tree
    rm -f "$index"
  }
  before=$(snapshot)
  just gen-lint
  after=$(snapshot)
  if [ "$before" != "$after" ]; then
    echo "just gen-lint changed these files:"
    git diff-tree -r --name-status "$before" "$after"
    exit 1
  fi

# Lints the GitHub Actions workflows like CI's "Lint GitHub Actions" job
lint-actions:
  actionlint -color

lint-fix:
  cargo clippy --fix
  cargo run -p rules_check
  bun oxlint --fix packages/

lint-ci-versions:
  rustc --version
  rustup --version
  cargo --version
  cargo sqlx --version
  cargo clippy --version
  bun oxlint --version

lint-ci:
  cargo sqlx prepare --check --workspace
  cargo clippy --fix
  cargo run -p rules_check
  bun oxlint --fix packages/

serve-docs:
    uv sync
    uv run zensical serve

# When you finished coding, run this command. Note that you should have already committed your changes.
# If you haven't run `sqlx prepare` at least once, you need to run `docker compose up`
# to lint the queries.
ready:
  git diff --exit-code --quiet
  cargo run -p xtask_codegen -- splinter
  cargo run -p xtask_codegen -- configuration
  cargo run -p docs_codegen
  cargo run -p xtask_codegen -- bindings
  cargo run -p xtask_codegen -- schema-types
  cargo sqlx prepare --workspace
  just format # format after codegen, so we don't have staged changes before lint-fix
  just lint-fix
  just format # format after linting so we format potential linting writes
  git diff --exit-code --quiet

# Creates a new crate
new-crate name:
  cargo new --lib crates/{{snakecase(name)}}
  cargo run -p xtask_codegen -- new-crate --name={{snakecase(name)}}

# Prints the treesitter tree of the given SQL file
tree-print file:
	cargo run --bin tree_print -- -f {{file}}

clear-branches:
    git branch --merged | egrep -v "(^\\*|main)" | xargs git branch -d

reset-git:
    git checkout main
    git pull
    just clear-branches

merge-main:
    git fetch origin main:main
    git merge main

quick-create branch commit:
    git checkout -b {{branch}}
    git add -A
    git commit -m "{{commit}}"
    git push
    gh pr create --fill

quick-modify:
    just format
    git add -A
    git commit -m "progress"
    git push

# Make sure to set your PGLS_LOG_PATH in your shell profile.
# You can use the PGLS_LOG_LEVEL to set your log level.
# We recommend to install `bunyan` (npm i -g bunyan) and pipe the output through there for color-coding:
# just show-logs | bunyan
show-logs:
    tail -f $(ls $PGLS_LOG_PATH/server.log.* | sort -t- -k2,2 -k3,3 -k4,4 | tail -n 1)
