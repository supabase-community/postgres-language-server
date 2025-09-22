insert into parted_conflict_test_1 values (2, 'b') on conflict (b) do update set a = excluded.a;
