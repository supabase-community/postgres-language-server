-- expect_lint/safety/changingColumnType
ALTER TABLE "core_recipe" ALTER COLUMN "edits" TYPE text USING "edits"::text;