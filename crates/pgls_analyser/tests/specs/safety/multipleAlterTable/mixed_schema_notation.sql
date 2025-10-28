-- expect_lint/safety/multipleAlterTable
-- Test mixing implicit and explicit public schema (should match)
ALTER TABLE authors ADD COLUMN name text;
ALTER TABLE public.authors ADD COLUMN email text;
