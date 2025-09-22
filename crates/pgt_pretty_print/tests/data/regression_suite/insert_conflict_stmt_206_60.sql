insert into parted_conflict_test values (3, 'a') on conflict (a) do update set b = excluded.b;
