create or replace function public.update_profile(
  handle app.valid_name,
  display_name text default null,
  bio text default null
)
returns void
language plpgsql
as $$
  declare
    v_is_org boolean;
  begin
    update app.accounts a
      set display_name = coalesce($2, a.display_name),
          bio = coalesce($3, a.bio)
    where a.handle = $1;

    update app.organizations o
      set display_name = coalesce($2, o.display_name),
          bio = coalesce($3, o.bio)
    where o.handle = $1;
  end;
$$;
