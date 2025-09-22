select to_tsvector('simple', 'q x') @@ 'q <-> (x | y <-> z)' AS "true";
