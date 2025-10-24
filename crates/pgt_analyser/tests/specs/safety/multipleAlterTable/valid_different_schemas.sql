-- Test ALTER TABLE on tables with same name but different schemas (should be safe)
-- expect_no_diagnostics
ALTER TABLE public.users ADD COLUMN age integer;
ALTER TABLE admin.users ADD COLUMN age integer;
