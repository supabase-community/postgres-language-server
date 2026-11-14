-- An output column can only be named on its own in ORDER BY, GROUP BY and DISTINCT ON. In an
-- expression, the name is a column.
-- expect_lint/requireColumnQualifier
select u.name as n from public.users u order by pg_catalog.lower(n);
select u.name as n from public.users u order by n;
select distinct on (n) u.name as n from public.users u;
