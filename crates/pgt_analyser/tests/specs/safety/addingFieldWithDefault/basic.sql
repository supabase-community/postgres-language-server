-- expect_lint/safety/addingFieldWithDefault
ALTER TABLE "core_recipe" ADD COLUMN "foo" integer DEFAULT 10;