FROM postgres:15

# Install build dependencies
RUN apt-get update && \
    apt-get install -y postgresql-server-dev-15 gcc make git && \
    cd /tmp && \
    git clone --branch v2.7.11 --depth 1 https://github.com/okbob/plpgsql_check.git && \
    cd plpgsql_check && \
    make && \
    make install && \
    apt-get remove -y postgresql-server-dev-15 gcc make git && \
    apt-get autoremove -y && \
    rm -rf /tmp/plpgsql_check /var/lib/apt/lists/*

# Add initialization script directly
RUN echo "CREATE EXTENSION IF NOT EXISTS plpgsql_check;" > /docker-entrypoint-initdb.d/01-create-extension.sql