insert into insertconflicttest values (26, 'Fig') on conflict (key) do update set fruit = excluded.fruit;
