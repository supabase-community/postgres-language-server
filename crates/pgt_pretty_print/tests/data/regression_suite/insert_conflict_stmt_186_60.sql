insert into selfconflict values (5,1), (5,2) on conflict(f1) do update set f2 = 0;
