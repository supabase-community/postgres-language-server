insert into insertconflicttest values (1, 'Apple') on conflict (keyy) do update set fruit = excluded.fruit;
