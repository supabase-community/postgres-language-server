insert into insertconflicttest values(0, 'Crowberry') on conflict (fruit, key, fruit, key) do nothing;
