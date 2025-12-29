FROM postgres:15

# Install build dependencies and extensions
RUN apt-get update && \
    apt-get install -y postgresql-server-dev-15 gcc make git && \
    # Install plpgsql_check
    cd /tmp && \
    git clone https://github.com/okbob/plpgsql_check.git && \
    cd plpgsql_check && \
    make && \
    make install && \
    # Install pglinter
    cd /tmp && \
    git clone https://github.com/pmpetit/pglinter.git && \
    cd pglinter && \
    make && \
    make install && \
    # Cleanup
    apt-get remove -y postgresql-server-dev-15 gcc make git && \
    apt-get autoremove -y && \
    rm -rf /tmp/plpgsql_check /tmp/pglinter /var/lib/apt/lists/*

# Add initialization script for extensions
RUN echo "CREATE EXTENSION IF NOT EXISTS plpgsql_check;" > /docker-entrypoint-initdb.d/01-create-extension.sql && \
    echo "CREATE EXTENSION IF NOT EXISTS pglinter;" >> /docker-entrypoint-initdb.d/01-create-extension.sql