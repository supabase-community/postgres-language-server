insert into insertconflicttest values (0, 'Bilberry') on conflict (key) do update set fruit = excluded.fruit;
