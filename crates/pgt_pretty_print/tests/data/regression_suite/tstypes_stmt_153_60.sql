select to_tsvector('simple', 'x y q y') @@ '!x <-> !!y' AS "true";
