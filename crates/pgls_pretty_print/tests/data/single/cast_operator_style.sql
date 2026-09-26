-- pgls-format: castStyle=operator
SELECT
  CAST(t.id AS bigint),
  CAST(t.name AS text),
  CAST(t.id AS public.object_id),
  CAST(nullif(t.a, '') AS date),
  CAST(t.a + t.b AS int),
  CAST(t.a || t.b AS text),
  CAST(CAST(t.a AS text) AS bigint),
  CAST((SELECT max(u.id) FROM s.u) AS bigint),
  CAST(CASE WHEN t.a THEN 1 ELSE 2 END AS text)
FROM s.t;
