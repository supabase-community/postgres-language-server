insert into inhpar as i values (3), (7) on conflict (f1)
  do update set (f1, f2) = (select i.f1, i.f2 || '+');
