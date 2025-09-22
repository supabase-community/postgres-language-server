SELECT d.oid, d.typname, d.typanalyze, t.oid, t.typname, t.typanalyze
FROM pg_type d JOIN pg_type t ON d.typbasetype = t.oid
WHERE d.typanalyze != t.typanalyze;
