insert into insertconflicttest values (1, 'Apple') on conflict do update set fruit = excluded.fruit;
