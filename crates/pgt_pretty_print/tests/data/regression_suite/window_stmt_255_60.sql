DELETE FROM empsalary RETURNING rank() OVER (ORDER BY random());
