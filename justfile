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
  cargo run -p xtask_codegen -- splinter
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
  cargo run -p xtask_codegen -- splinter
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

# === Pretty Printer Development ===

# Run pretty printer agentic task (Stop hook auto-loops until tests pass)
pp-agentic:
    claude --dangerously-skip-permissions "Read agentic/pretty_printer.md and agentic/session_log.md. \
    \
    Your goal: Complete the pretty printer by fixing node implementations until ALL tests pass. \
    \
    Workflow: \
    1. Run 'just pp-status' to see current state \
    2. Run 'just pp-failing' to find failing tests \
    3. Pick a failing test and debug with 'just pp-debug <name>' \
    4. Fix the emit_* function in crates/pgls_pretty_print/src/nodes/*.rs \
    5. Verify with 'just pp-test <pattern>' \
    6. Accept valid snapshots with 'just pp-review' \
    7. Repeat \
    \
    Follow the Implementation Learnings in pretty_printer.md. Update session_log.md with your progress."

# Show pretty printer implementation status
pp-status:
    @./scripts/pp-status.sh

# Test with pattern filter (e.g., just pp-test select_stmt)
pp-test pattern:
    cargo test -p pgls_pretty_print -- {{pattern}} --show-output

# List failing tests
pp-failing:
    @cargo test -p pgls_pretty_print 2>&1 | grep "FAILED" | head -30

# Debug a specific test with full output
pp-debug name:
    cargo test -p pgls_pretty_print {{name}} -- --show-output --nocapture

# Review pending snapshots
pp-review:
    cargo insta review -p pgls_pretty_print

# Accept all pending snapshots
pp-accept:
    cargo insta accept -p pgls_pretty_print

# Analyze failure patterns
pp-analyze:
    @echo "=== Failure Analysis ===" && \
    cargo test -p pgls_pretty_print 2>&1 | grep -oE "test_(single|multi)__[a-z0-9_]+" | sort | uniq -c | sort -rn | head -20

# Run only single-statement tests (faster iteration)
pp-single:
    cargo test -p pgls_pretty_print test_single

# Run only multi-statement tests
pp-multi:
    cargo test -p pgls_pretty_print test_multi

# Short aliases (only for commands without required args)
pps: pp-status
ppf: pp-failing
ppr: pp-review

# ============================================================================
# WASM Build
# ============================================================================

# Build WASM bindings (debug) - uses Nix if available
build-wasm:
    #!/usr/bin/env bash
    if command -v nix &> /dev/null && [ -f crates/pgls_wasm/flake.nix ]; then
        echo "Building with Nix..."
        nix develop ./crates/pgls_wasm#default --command ./crates/pgls_wasm/build-wasm.sh
    else
        ./crates/pgls_wasm/build-wasm.sh
    fi

# Build WASM bindings (release) - uses Nix if available
build-wasm-release:
    #!/usr/bin/env bash
    if command -v nix &> /dev/null && [ -f crates/pgls_wasm/flake.nix ]; then
        echo "Building with Nix..."
        nix develop ./crates/pgls_wasm#default --command ./crates/pgls_wasm/build-wasm.sh --release
    else
        ./crates/pgls_wasm/build-wasm.sh --release
    fi

# Build WASM using Nix (recommended)
build-wasm-nix:
    nix build ./crates/pgls_wasm#default

# Enter WASM development shell with Nix
wasm-shell:
    nix develop ./crates/pgls_wasm#default

# Check if WASM build prerequisites are installed
check-wasm-prereqs:
    @echo "Checking WASM build prerequisites..."
    @command -v nix >/dev/null 2>&1 && echo "✓ Nix found (recommended)" || echo "○ Nix not found (optional but recommended)"
    @command -v emcc >/dev/null 2>&1 && echo "✓ Emscripten (emcc) found" || echo "✗ Emscripten not found - install from https://emscripten.org or use Nix"
    @rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-emscripten && echo "✓ wasm32-unknown-emscripten target installed" || echo "✗ Missing target - run: rustup target add wasm32-unknown-emscripten"

# Install WASM build prerequisites (non-Nix)
install-wasm-prereqs:
    rustup target add wasm32-unknown-emscripten
    @echo ""
    @echo "NOTE: You also need to install Emscripten SDK manually:"
    @echo "  https://emscripten.org/docs/getting_started/downloads.html"
    @echo ""
    @echo "After installing, activate it with:"
    @echo "  source /path/to/emsdk/emsdk_env.sh"
    @echo ""
    @echo "Or use Nix (recommended): just wasm-shell"

