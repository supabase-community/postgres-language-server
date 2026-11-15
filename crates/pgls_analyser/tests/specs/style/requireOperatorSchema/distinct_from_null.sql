-- IS [NOT] DISTINCT FROM with a bare NULL is a null test, which looks up no operator.
-- expect_no_diagnostics
select '{}'::pg_catalog.json is distinct from null;
select null is not distinct from p.payload from public.payloads p;
