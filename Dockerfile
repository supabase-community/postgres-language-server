FROM postgres:15

# Install build dependencies
RUN apt-get update && \
    apt-get install -y postgresql-server-dev-15 gcc make git curl pkg-config libssl-dev libclang-dev clang libicu-dev openssl && \
    mkdir -p /etc/postgresql/ssl && \
    openssl req -x509 -newkey rsa:2048 -sha256 -days 3650 -nodes \
      -subj "/CN=localhost" \
      -keyout /etc/postgresql/ssl/server.key \
      -out /etc/postgresql/ssl/server.crt && \
    chmod 600 /etc/postgresql/ssl/server.key && \
    chmod 644 /etc/postgresql/ssl/server.crt && \
    chown postgres:postgres /etc/postgresql/ssl/server.key /etc/postgresql/ssl/server.crt && \
    # Install plpgsql_check (C extension - simple make install)
    # Pin to v2.7.11 for stability with PG15
    cd /tmp && \
    git clone --branch v2.7.11 --depth 1 https://github.com/okbob/plpgsql_check.git && \
    cd plpgsql_check && \
    make && \
    make install && \
    cd /tmp && \
    rm -rf /tmp/plpgsql_check && \
    # Install Rust for pglinter (pgrx-based extension)
    # pgrx 0.18.0 requires Rust 1.89+
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.89.0 && \
    . $HOME/.cargo/env && \
    # Install cargo-pgrx (version must match pglinter's pgrx dependency)
    cargo install cargo-pgrx --version 0.18.0 --locked && \
    # Initialize pgrx for PostgreSQL 15
    cargo pgrx init --pg15 $(which pg_config) && \
    # Clone and build pglinter (requires v1.1.0+ for get_violations API + rule_messages table)
    cd /tmp && \
    git clone --depth 1 https://github.com/pmpetit/pglinter.git && \
    cd pglinter && \
    cargo pgrx install --pg-config $(which pg_config) --release && \
    # Cleanup Rust toolchains and temporary sources
    rm -rf /tmp/pglinter $HOME/.cargo $HOME/.rustup && \
    rm -rf /var/lib/apt/lists/*

RUN printf "\nssl = on\nssl_cert_file = '/etc/postgresql/ssl/server.crt'\nssl_key_file = '/etc/postgresql/ssl/server.key'\n" >> /usr/share/postgresql/postgresql.conf.sample

# Add initialization script for extensions
# Create extensions in a dedicated 'extensions' schema to avoid triggering extensionInPublic lint
# Create in template1 so they're available in all new databases (for SQLx tests)
# Also create in postgres database for direct connections
RUN echo "\\c template1" > /docker-entrypoint-initdb.d/01-create-extension.sql && \
    echo "CREATE SCHEMA IF NOT EXISTS extensions;" >> /docker-entrypoint-initdb.d/01-create-extension.sql && \
    echo "CREATE EXTENSION IF NOT EXISTS plpgsql_check SCHEMA extensions;" >> /docker-entrypoint-initdb.d/01-create-extension.sql && \
    echo "CREATE EXTENSION IF NOT EXISTS pglinter SCHEMA extensions;" >> /docker-entrypoint-initdb.d/01-create-extension.sql && \
    echo "\\c postgres" >> /docker-entrypoint-initdb.d/01-create-extension.sql && \
    echo "CREATE SCHEMA IF NOT EXISTS extensions;" >> /docker-entrypoint-initdb.d/01-create-extension.sql && \
    echo "CREATE EXTENSION IF NOT EXISTS plpgsql_check SCHEMA extensions;" >> /docker-entrypoint-initdb.d/01-create-extension.sql && \
    echo "CREATE EXTENSION IF NOT EXISTS pglinter SCHEMA extensions;" >> /docker-entrypoint-initdb.d/01-create-extension.sql
