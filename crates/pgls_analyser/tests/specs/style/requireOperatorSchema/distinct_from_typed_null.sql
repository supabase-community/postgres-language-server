-- A NULL with a cast is not a bare NULL, so = is looked up.
-- expect_lint/requireOperatorSchema
select p.payload is distinct from null::pg_catalog.jsonb from public.payloads p;
