insert into insertconflicttest values (25, 'Fig') on conflict (fruit) do update set fruit = excluded.fruit;
