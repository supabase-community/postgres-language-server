insert into insertconflict values (1, 2)
on conflict (coalesce(a, 0)) do nothing;
