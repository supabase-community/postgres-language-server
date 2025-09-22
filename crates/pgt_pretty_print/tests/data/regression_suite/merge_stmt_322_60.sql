INSERT INTO pa_target SELECT id, id * 100, 'initial' FROM generate_series(1,15,2) AS id;
