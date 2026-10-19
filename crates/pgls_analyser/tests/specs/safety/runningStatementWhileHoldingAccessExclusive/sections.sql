-- expect_no_diagnostics
-- The lock of the up section isn't held in the down section.
-- section: up
ALTER TABLE authors ADD COLUMN email text;

-- section: down
UPDATE authors SET name = 'x';

-- Without a transaction, each statement releases its locks.
-- section: up transaction:false
ALTER TABLE books ADD COLUMN isbn text;
UPDATE books SET isbn = '';

-- section: down
ALTER TABLE books DROP COLUMN isbn;
