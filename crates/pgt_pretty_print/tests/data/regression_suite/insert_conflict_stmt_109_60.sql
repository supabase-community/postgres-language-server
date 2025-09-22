insert into syscolconflicttest values (1) on conflict (key) do update set data = excluded.ctid::text;
