-- expect_lint/safety/multipleAlterTable
-- Test multiple ALTER TABLE statements with explicit schema
ALTER TABLE public.users ADD COLUMN age integer;
ALTER TABLE public.users ADD COLUMN country text;
