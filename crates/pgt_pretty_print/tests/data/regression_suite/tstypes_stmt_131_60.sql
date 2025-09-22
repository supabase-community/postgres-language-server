select to_tsvector('simple', 'q y z') @@ 'q <-> (x | y <-> z)' AS "true";
