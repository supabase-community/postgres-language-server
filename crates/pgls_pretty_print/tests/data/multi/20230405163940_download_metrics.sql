-- Return API request headers
create or replace function app.api_request_headers()
    returns json
    language sql
    stable
    as
$$
    select coalesce(current_setting('request.headers', true)::json, '{}'::json);
$$;


-- Return specific API request header
create or replace function app.api_request_header(text)
    returns text
    language sql
    stable
    as
$$
    select app.api_request_headers() ->> $1
$$;


-- IP address of current API request
create or replace function app.api_request_ip()
    returns inet
    language sql
    stable
    as
$$
    select split_part(app.api_request_header('x-forwarded-for') || ',', ',', 1)::inet
$$;

-- IP address of current API request
create or replace function app.api_request_client_info()
    returns text
    language sql
    stable
    as
$$
    select app.api_request_header('x-client-info')
$$;


create table app.downloads(
    id uuid primary key default gen_random_uuid(),
    package_id uuid not null references app.packages(id),
    ip inet not null default app.api_request_ip()::inet,
    client_info text default app.api_request_client_info(),
    created_at timestamptz not null default now()
);

-- Speed up metrics query
create index downloads_package_id_ip
  on app.downloads (package_id);

create index downloads_created_at
  on app.downloads
  using brin(created_at);

create or replace function public.register_download(package_name text)
    returns void
    language sql
    security definer
    as
$$
  insert into app.downloads(package_id)
  select id
  from app.packages ap
  where ap.package_name = $1
$$;


-- Public facing download metrics view. For website only. Not a stable part of dbdev API
create materialized view public.download_metrics
as
  select
    dl.package_id,
    count(dl.id) downloads_all_time,
    count(dl.id) filter (where dl.created_at > now() - '180 days'::interval) downloads_180_days,
    count(dl.id) filter (where dl.created_at > now() - '90 days'::interval) downloads_90_days,
    count(dl.id) filter (where dl.created_at > now() - '30 days'::interval) downloads_30_day
  from
    app.downloads dl
  group by
      dl.package_id;


-- High frequency refresh for debugging
select cron.schedule(
  'refresh download metrics',
  '*/30 * * * *',
  'refresh materialized view public.download_metrics;'
);
