insert into insertconflicttest values(0, 'Crowberry') on conflict (key, fruit) do nothing;
