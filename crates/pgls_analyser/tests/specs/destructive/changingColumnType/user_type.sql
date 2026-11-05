-- myschema.text is a type of the user, and text[] isn't binary compatible with text.
-- expect_lint/changingColumnType
-- expect_lint/changingColumnType
ALTER TABLE "core_recipe" ALTER COLUMN "edits" TYPE myschema.text;
ALTER TABLE "core_recipe" ALTER COLUMN "tags" TYPE text[];
