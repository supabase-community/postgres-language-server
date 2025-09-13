-- https://postgrestools.com/analyser/safety/addingNotNullField

-- Should trigger: Setting column NOT NULL (in Postgres < 11)
ALTER TABLE "core_recipe" ALTER COLUMN "foo" SET NOT NULL;