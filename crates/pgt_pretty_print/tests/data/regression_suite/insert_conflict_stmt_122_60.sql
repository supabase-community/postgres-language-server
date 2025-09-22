insert into insertconflictv values (1,'bar')
  on conflict (f1) do update set f2 = excluded.f2;
