-- A NOT ENFORCED foreign key (Postgres 18) is never validated
ALTER TABLE email ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users (id) NOT ENFORCED;
ALTER TABLE email ADD COLUMN owner_id int REFERENCES users (id) NOT ENFORCED;
ALTER TABLE email ADD COLUMN sender_id int REFERENCES users (id) DEFERRABLE NOT ENFORCED;
ALTER TABLE email ALTER CONSTRAINT fk_user NOT ENFORCED;
ALTER TABLE email ALTER CONSTRAINT fk_user DEFERRABLE;
-- expect_lint/addingForeignKeyConstraint
ALTER TABLE email ADD COLUMN reader_id int REFERENCES users (id) ENFORCED;
-- expect_lint/addingForeignKeyConstraint
ALTER TABLE email ADD COLUMN editor_id int REFERENCES users (id) NOT ENFORCED CHECK (editor_id > 0) REFERENCES users (id);
-- Enforcing it checks all rows
-- expect_lint/addingForeignKeyConstraint
ALTER TABLE email ALTER CONSTRAINT fk_user ENFORCED;
