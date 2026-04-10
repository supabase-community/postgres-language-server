create or replace function public.download_metrics(public.packages)
returns setof public.download_metrics rows 1
language sql stable
as $$
  select *
  from public.download_metrics dm
  where dm.package_id = $1.id;
$$;
