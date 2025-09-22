SELECT size, pg_size_pretty(size), pg_size_pretty(-1 * size) FROM
    (VALUES (10239::bigint), (10240::bigint),
            (10485247::bigint), (10485248::bigint),
            (10736893951::bigint), (10736893952::bigint),
            (10994579406847::bigint), (10994579406848::bigint),
            (11258449312612351::bigint), (11258449312612352::bigint)) x(size);
