-- sectionMarkers: dbmate
-- expect_no_diagnostics
-- The lock of the up section isn't held in the down section.
-- migrate:up
ALTER TABLE authors ADD COLUMN email text;

-- migrate:down
UPDATE authors SET name = 'x';

-- Without a transaction, each statement releases its locks.
-- migrate:up transaction:false
ALTER TABLE books ADD COLUMN isbn text;
UPDATE books SET isbn = '';

-- migrate:down
ALTER TABLE books DROP COLUMN isbn;
