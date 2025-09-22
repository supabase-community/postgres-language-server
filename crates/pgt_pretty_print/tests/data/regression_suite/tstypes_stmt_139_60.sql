select to_tsvector('simple', 'y z q') @@ '(x | y <-> z) <-> q' AS "true";
