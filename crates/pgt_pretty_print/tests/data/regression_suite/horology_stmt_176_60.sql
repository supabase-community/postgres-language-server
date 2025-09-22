SELECT t.f1 AS t, i.f1 AS i, t.f1 + i.f1 AS "add", t.f1 - i.f1 AS "subtract"
  FROM TIME_TBL t, INTERVAL_TBL i
  WHERE isfinite(i.f1)
  ORDER BY 1,2;
