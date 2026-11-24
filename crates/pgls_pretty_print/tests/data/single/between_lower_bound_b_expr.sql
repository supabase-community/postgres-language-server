SELECT true BETWEEN ('foo' LIKE ANY (ARRAY['foo'])) AND true, a BETWEEN (b IS NULL) AND c, a BETWEEN b + 1 AND c FROM t;
