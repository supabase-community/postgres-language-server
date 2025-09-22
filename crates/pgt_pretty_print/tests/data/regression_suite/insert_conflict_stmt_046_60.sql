insert into insertconflicttest values (3, 'Kiwi') on conflict (key, fruit) do update set insertconflicttest.fruit = 'Mango';
