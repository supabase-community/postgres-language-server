insert into insertconflicttest values (29, 'Nectarine') on conflict (key) do update set fruit = excluded.fruit;
