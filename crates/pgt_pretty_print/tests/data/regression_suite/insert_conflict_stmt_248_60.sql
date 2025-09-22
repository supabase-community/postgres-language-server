insert into parted_conflict values (50, 'cincuenta', 2)
  on conflict (a, b) do update set (a, b, c) = row(excluded.*)
  where parted_conflict = (50, text 'cincuenta', 1) and
        excluded = (50, text 'cincuenta', 2);
