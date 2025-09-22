select to_tsvector('simple', 'x q') @@ '(x | y <-> !z) <-> q' AS "true";
