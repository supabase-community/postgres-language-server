insert into insertconflicttest values (15, 'Cranberry') on conflict (key) do update set fruit = excluded.fruit;
