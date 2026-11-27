# The major version of the test database, e.g. `docker build --build-arg PG_MAJOR=18 .`.
ARG PG_MAJOR=15
FROM postgres:${PG_MAJOR}

# plpgsql_check from the PostgreSQL apt repository, which the image already uses, and pglinter from
# its release packages, so that nothing is compiled here.
ARG PG_MAJOR
ARG TARGETARCH
ARG PGLINTER_VERSION=2.0.0
ADD https://github.com/pmpetit/pglinter/releases/download/${PGLINTER_VERSION}/postgresql_pglinter_${PG_MAJOR}_${PGLINTER_VERSION}_${TARGETARCH}.deb /tmp/pglinter.deb
RUN apt-get update && \
    apt-get install -y --no-install-recommends openssl "postgresql-${PG_MAJOR}-plpgsql-check" /tmp/pglinter.deb && \
    rm -rf /var/lib/apt/lists/* /tmp/pglinter.deb

RUN mkdir -p /etc/postgresql/ssl && \
    openssl req -x509 -newkey rsa:2048 -sha256 -days 3650 -nodes \
      -subj "/CN=localhost" \
      -keyout /etc/postgresql/ssl/server.key \
      -out /etc/postgresql/ssl/server.crt && \
    chmod 600 /etc/postgresql/ssl/server.key && \
    chmod 644 /etc/postgresql/ssl/server.crt && \
    chown postgres:postgres /etc/postgresql/ssl/server.key /etc/postgresql/ssl/server.crt

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

# Several worktrees run their tests against one database at the same time. Each test of the
# workspace and the language server opens up to 20 connections of its own, so the default of 100
# runs out and the tests fail with `PoolTimedOut`. A command-line setting also applies to a
# database volume that was initialised by an older image.
CMD ["postgres", "-c", "max_connections=500"]
