DELETE FROM empsalary WHERE (rank() OVER (ORDER BY random())) > 10;
