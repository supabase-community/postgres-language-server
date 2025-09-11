-- https://postgrestools.com/analyser/safety/addingForeignKeyConstraint

-- Should trigger: Adding constraint without NOT VALID
ALTER TABLE "email" ADD CONSTRAINT "fk_user" FOREIGN KEY ("user_id") REFERENCES "user" ("id");