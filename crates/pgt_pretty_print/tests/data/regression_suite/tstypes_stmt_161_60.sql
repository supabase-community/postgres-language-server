select to_tsvector('simple', 'x y q y') @@ '!foo' AS "true";
