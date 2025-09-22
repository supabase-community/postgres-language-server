insert into parted_conflict_test (a, b) values (1, 'a'), (2, 'a'), (4, 'a') on conflict (a) do update set b = excluded.b where excluded.b = 'b';
