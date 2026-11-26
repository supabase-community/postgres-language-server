-- IS binds less tightly than every operator but AND, OR and NOT, and is non-associative: without
-- parentheses, `a IS DISTINCT FROM b IS NULL` is a syntax error and `NOT a IS NULL` tests `a`.
SELECT (a IS DISTINCT FROM b) IS NULL, (a IS NOT DISTINCT FROM b) IS TRUE, (a AND b) IS NOT NULL, (NOT a) IS NULL, (a OR b) IS UNKNOWN FROM t;
