create or replace function public.popular_packages()
returns setof public.packages
language sql stable
as $$
  select * from public.packages p
  order by (
    select (dm.downloads_30_day * 5) + (dm.downloads_90_days * 2) + dm.downloads_180_days
    from public.download_metrics dm
    where dm.package_id = p.id
  ) desc nulls last, p.created_at desc;
$$;
