SELECT * FROM empsalary WHERE row_number() OVER (ORDER BY salary) < 10;
