SELECT (a = f(x)) = ANY (SELECT true), (a = b) = ALL (ARRAY[true]), (a LIKE b) LIKE c, a BETWEEN (b = c) AND (d AND e), (a IS DISTINCT FROM b) IS DISTINCT FROM c, a + (b = ANY (SELECT 1)) FROM t;
