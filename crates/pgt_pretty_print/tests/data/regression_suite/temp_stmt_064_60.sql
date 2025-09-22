create function public.whoami() returns text
  as $$select 'public'::text$$ language sql;
