FROM postgres:15

# Install build dependencies
RUN apt-get update && \
    apt-get install -y postgresql-server-dev-15 gcc make git curl pkg-config libssl-dev libclang-dev clang && \
    # Install plpgsql_check (C extension - simple make install)
    cd /tmp && \
    git clone https://github.com/okbob/plpgsql_check.git && \
    cd plpgsql_check && \
    make && \
    make install && \
    cd /tmp && \
    rm -rf /tmp/plpgsql_check && \
    # Install Rust for pglinter (pgrx-based extension)
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env && \
    # Install cargo-pgrx (version must match pglinter's pgrx dependency)
    cargo install cargo-pgrx --version 0.16.1 --locked && \
    # Initialize pgrx for PostgreSQL 15
    cargo pgrx init --pg15 $(which pg_config) && \
    # Clone and build pglinter (using feat/83/violation_list branch for get_violations API)
    cd /tmp && \
    git clone -b feat/83/violation_list https://github.com/pmpetit/pglinter.git && \
    cd pglinter && \
    cargo pgrx install --pg-config $(which pg_config) --release && \
    # Cleanup Rust and build dependencies
    rm -rf /tmp/pglinter $HOME/.cargo $HOME/.rustup && \
    apt-get remove -y gcc make git curl pkg-config libssl-dev libclang-dev clang && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

# Add initialization script for extensions
# Only create in postgres database (NOT template1) to avoid polluting test databases
# Tests that need extensions can create them explicitly
RUN printf '%s\n' \
    "CREATE SCHEMA IF NOT EXISTS extensions;" \
    "CREATE EXTENSION IF NOT EXISTS plpgsql_check SCHEMA extensions;" \
    "CREATE EXTENSION IF NOT EXISTS pglinter SCHEMA extensions;" \
    > /docker-entrypoint-initdb.d/01-create-extension.sql