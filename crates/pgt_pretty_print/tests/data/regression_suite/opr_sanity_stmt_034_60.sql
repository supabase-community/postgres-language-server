SELECT oid::regprocedure, proargmodes, provariadic
FROM pg_proc
WHERE (proargmodes IS NOT NULL AND 'v' = any(proargmodes))
    IS DISTINCT FROM
    (provariadic != 0);
