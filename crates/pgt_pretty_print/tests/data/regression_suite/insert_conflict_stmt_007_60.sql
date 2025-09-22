insert into insertconflicttest values(0, 'Crowberry') on conflict (fruit) do nothing;
