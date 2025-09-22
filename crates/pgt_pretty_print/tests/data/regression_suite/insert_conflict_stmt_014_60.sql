insert into insertconflicttest values(0, 'Crowberry') on conflict (key, fruit collate "C") do nothing;
