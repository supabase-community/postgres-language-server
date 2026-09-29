-- expect_no_diagnostics
ALTER TABLE "core_recipe" ALTER COLUMN "edits" TYPE text;
ALTER TABLE "core_recipe" ALTER COLUMN "name" TYPE varchar;
ALTER TABLE "core_recipe" ALTER COLUMN "amount" TYPE numeric;
