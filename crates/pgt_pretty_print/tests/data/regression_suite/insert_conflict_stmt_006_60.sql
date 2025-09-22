insert into insertconflicttest values(0, 'Crowberry') on conflict (key) do nothing;
