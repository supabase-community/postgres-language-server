INSERT INTO pa_source SELECT id, id * 10  FROM generate_series(1,14) AS id;
