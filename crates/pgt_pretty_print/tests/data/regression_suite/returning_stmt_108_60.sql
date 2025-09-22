INSERT INTO zerocol SELECT RETURNING old.*, new.*, *;
