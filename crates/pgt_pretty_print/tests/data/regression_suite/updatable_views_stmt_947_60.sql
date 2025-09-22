create function base_tab_def_view_instrig_func() returns trigger
as
$$
begin
  insert into base_tab_def values (new.a, new.b, new.c, new.d, new.e);
  return new;
end;
$$
language plpgsql;
