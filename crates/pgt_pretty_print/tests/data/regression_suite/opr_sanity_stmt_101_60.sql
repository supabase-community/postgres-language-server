SELECT oid, proname
FROM pg_proc AS p
WHERE prokind = 'a' AND proargdefaults IS NOT NULL;
