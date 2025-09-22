SELECT size, pg_size_pretty(size), pg_size_pretty(-1 * size) FROM
    (VALUES (10239::numeric), (10240::numeric),
            (10485247::numeric), (10485248::numeric),
            (10736893951::numeric), (10736893952::numeric),
            (10994579406847::numeric), (10994579406848::numeric),
            (11258449312612351::numeric), (11258449312612352::numeric),
            (11528652096115048447::numeric), (11528652096115048448::numeric)) x(size);
