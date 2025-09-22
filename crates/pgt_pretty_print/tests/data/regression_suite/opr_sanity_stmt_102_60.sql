SELECT p.oid, proname
FROM pg_proc AS p JOIN pg_aggregate AS a ON a.aggfnoid = p.oid
WHERE prokind = 'a' AND provariadic != 0 AND a.aggkind = 'n';
