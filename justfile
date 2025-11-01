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
	cargo install tree-sitter-cli
	cargo binstall cargo-insta taplo-cli sqlx-cli
	cargo binstall --git "https://github.com/astral-sh/uv" uv
	bun install

# Upgrades the tools needed to develop
upgrade-tools:
	cargo install cargo-binstall --force
	cargo install tree-sitter-cli  --force
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

# Run tests for the crate passed as argument e.g. just test-create pgls_cli
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
  cargo sqlx prepare --workspace
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

# Make sure to set your PGLS_LOG_PATH in your shell profile.
# You can use the PGLS_LOG_LEVEL to set your log level.
# We recommend to install `bunyan` (npm i -g bunyan) and pipe the output through there for color-coding:
# just show-logs | bunyan
show-logs:
    tail -f $(ls $PGLS_LOG_PATH/server.log.* | sort -t- -k2,2 -k3,3 -k4,4 | tail -n 1)

# Run a codex agent with the given agentic prompt file.
# Commented out by default to avoid accidental usage that may incur costs.
agentic name:
    codex exec --yolo "please read agentic/{{name}}.md and follow the instructions closely while continueing the described task. Make sure to understand recent Session History, Implementation Learnings and read all instructions. Continue until the task is complete."

agentic-loop name:
    #!/usr/bin/env bash
    set +e  # Don't exit on error
    echo "Starting agentic loop - will retry on rate limits..."
    echo "Stop keyword: ===AGENTIC_TASK_COMPLETE==="
    iteration=1
    output_file=$(mktemp)
    trap "rm -f $output_file" EXIT

    while true; do
        echo "$(date): Starting iteration $iteration..."

        # Run agentic and capture output
        just agentic {{name}} 2>&1 | tee "$output_file"
        exit_code=${PIPESTATUS[0]}

        # Check for completion keyword in last 10 lines only
        if tail -n 10 "$output_file" | grep -q "===AGENTIC_TASK_COMPLETE==="; then
            echo "$(date): ✓ Task complete keyword detected - stopping loop"
            break
        fi

        # Handle exit codes
        if [ $exit_code -eq 0 ]; then
            echo "$(date): Iteration $iteration completed successfully!"
            iteration=$((iteration + 1))
        elif [ $exit_code -eq 1 ]; then
            echo "$(date): Rate limit hit (exit code 1) - waiting 3 hours before retry..."
            sleep 10800  # 3 hours = 10800 seconds
            echo "$(date): Resuming after 3-hour wait..."
        else
            echo "$(date): Unexpected error (exit code $exit_code) - stopping loop"
            break
        fi
    done

    rm -f "$output_file"
    echo "$(date): Agentic loop finished after $iteration iterations"

