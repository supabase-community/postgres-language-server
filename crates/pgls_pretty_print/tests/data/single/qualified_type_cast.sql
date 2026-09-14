SELECT
	CAST(t.id AS public.object_id),
	CAST(t.ids AS public.object_id[]),
	CAST(t.n AS bigint)
FROM s.t;
