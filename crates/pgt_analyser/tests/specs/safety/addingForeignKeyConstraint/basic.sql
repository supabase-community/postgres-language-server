-- expect_lint/safety/addingForeignKeyConstraint
ALTER TABLE "email" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id");
