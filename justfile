_default:
  just --list -u

alias f := format
alias r := ready
alias l := lint
alias t := test
alias rg := reset-git
alias qm := quick-modify

# Installs the tools needed to develop
install-tools:
	cargo install cargo-binstall
	cargo binstall cargo-insta taplo-cli sqlx-cli
	cargo binstall --git "https://github.com/astral-sh/uv" uv
	bun install

# Upgrades the tools needed to develop
upgrade-tools:
	cargo install cargo-binstall --force
	cargo binstall cargo-insta taplo-cli sqlx-cli --force
	cargo binstall --git "https://github.com/astral-sh/uv" uv --force

# Generates code generated files for the linter
gen-lint:
  cargo run -p xtask_codegen -- analyser
  cargo run -p xtask_codegen -- configuration
  cargo run -p xtask_codegen -- bindings
  cargo run -p rules_check
  cargo run -p docs_codegen
  just format

# Fix cargo check issues using Claude
fix-cargo-check:
  unset ANTHROPIC_API_KEY && claude --dangerously-skip-permissions -p "Please run cargo check and fix any issues. It is okay to remove things that are not fully implemented yet."

# Run the autonomous pretty print implementation generator
pretty-print: fix-cargo-check
  cargo xtask agentic

# Run the autonomous pretty print implementation generator with infinite retry loop
pretty-print-forever: fix-cargo-check
  #!/bin/bash
  echo "Starting autonomous pretty print generator with error recovery..."
  while true; do
    echo "$(date): Starting agentic loop iteration..."
    if cargo xtask agentic; then
      echo "$(date): Agentic loop completed successfully!"
      break
    else
      echo "$(date): Agentic loop failed, waiting 6 hours 1 minute before retry..."
      sleep 3660  # 1 hour 1 minute = 1 * 60 * 60 + 1 * 60 = 3600 + 60 = 3660 seconds
      echo "$(date): Resuming after 1-hour 1-minute wait..."
    fi
  done

fix-and-push: fix-cargo-check
  just quick-modify

# Creates a new lint rule in the given path, with the given name. Name has to be camel case. Group should be lowercase.
new-lintrule group rulename severity="error":
  cargo run -p xtask_codegen -- new-lintrule --category=lint --name={{rulename}} --group={{group}} --severity={{severity}}
  just gen-lint

# Format Rust, JS and TOML files
format:
	cargo fmt
	taplo format
	bun biome format --write

format-ci:
	cargo fmt --all --check
	taplo format --check
	bun biome format

format-ci-versions:
	cargo --version
	taplo --version
	echo "Biome $(bun biome --version)"

[unix]
_touch file:
  touch {{file}}

[windows]
_touch file:
  (gci {{file}}).LastWriteTime = Get-Date

# Run tests of all crates
test:
	cargo test run --no-fail-fast

# Run tests for the crate passed as argument e.g. just test-create pgt_cli
test-crate name:
	cargo test run -p {{name}} --no-fail-fast

# Run doc tests
test-doc:
	cargo test --doc

# Alias for `cargo clippy`, it runs clippy on the whole codebase
lint:
  cargo clippy
  cargo run -p rules_check
  bun biome lint

lint-fix:
  cargo clippy --fix
  cargo run -p rules_check
  bun biome lint --write

lint-ci-versions:
  rustc --version
  rustup --version
  cargo --version
  cargo sqlx --version
  cargo clippy --version
  echo "Biome $(bun biome --version)"

lint-ci:
  cargo sqlx prepare --check --workspace
  cargo clippy --fix
  cargo run -p rules_check
  bun biome lint --write

serve-docs:
    uv sync
    uv run mkdocs serve

# When you finished coding, run this command. Note that you should have already committed your changes.
# If you haven't run `sqlx prepare` at least once, you need to run `docker compose up`
# to lint the queries.
ready:
  git diff --exit-code --quiet
  cargo run -p xtask_codegen -- configuration
  cargo run -p docs_codegen
  cargo run -p xtask_codegen -- bindings
  just lint-fix
  just format
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

# Run a claude agent with the given agentic prompt file.
# Commented out by default to avoid accidental usage that may incur costs.
agentic name:
    unset ANTHROPIC_API_KEY && claude --dangerously-skip-permissions -p "please read agentic/{{name}}.md and follow the instructions closely"

# Run agentic loop that breaks when task is complete
agentic-loop name:
    #!/usr/bin/env bash
    while true; do
        echo "$(date): Calling Claude..."
        output=$(just agentic {{name}} 2>&1)
        exit_code=$?
        echo "$output"

        if [ $exit_code -ne 0 ]; then
            echo "$(date): Error occurred, waiting 61 minutes before retry..."
            sleep 3660  # 61 minutes
            continue
        fi

        if echo "$output" | grep -q "TASK COMPLETE"; then
            echo "$(date): Task completed!"
            break
        fi
    done

# Make sure to set your PGT_LOG_PATH in your shell profile.
# You can use the PGT_LOG_LEVEL to set your log level.
# We recommend to install `bunyan` (npm i -g bunyan) and pipe the output through there for color-coding:
# just show-logs | bunyan
show-logs:
    tail -f $(ls $PGT_LOG_PATH/server.log.* | sort -t- -k2,2 -k3,3 -k4,4 | tail -n 1)
