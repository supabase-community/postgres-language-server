select to_tsvector('simple', 'q x y') @@ 'q <-> (x | y <-> z)' AS "true";
