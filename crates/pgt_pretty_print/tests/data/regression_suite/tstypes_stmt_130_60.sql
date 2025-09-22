select to_tsvector('simple', 'q y') @@ 'q <-> (x | y <-> z)' AS "false";
