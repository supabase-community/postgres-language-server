insert into storage.buckets (id, name)
values
    ('package_versions', 'package_versions'),
    ('package_upgrades', 'package_upgrades');

create function app.to_package_name(handle app.valid_name, partial_name app.valid_name)
    returns text
    immutable
    language sql
as $$
    select format('%s-%s', $1, $2)
$$;

create table app.packages(
    id uuid primary key default gen_random_uuid(),
    package_name text not null generated always as (app.to_package_name(handle, partial_name)) stored,
    handle app.valid_name not null references app.handle_registry(handle),
    partial_name app.valid_name not null, -- ex: math
    control_description varchar(1000),
    control_relocatable bool not null default false,
    control_requires varchar(128)[] default '{}'::varchar(128)[],
    created_at timestamptz not null default now(),
    unique (handle, partial_name)
);
create index packages_partial_name_search_idx on app.packages using gin (partial_name extensions.gin_trgm_ops);
create index packages_handle_search_idx on app.packages using gin (handle extensions.gin_trgm_ops);

create table app.package_versions(
    id uuid primary key default gen_random_uuid(),
    package_id uuid not null references app.packages(id),
    version_struct app.semver not null,
    version text not null generated always as (app.semver_to_text(version_struct)) stored,
    sql varchar(250000),
    description_md varchar(250000),
    created_at timestamptz not null default now(),
    unique(package_id, version_struct)
);

create table app.package_upgrades(
    id uuid primary key default gen_random_uuid(),
    package_id uuid not null references app.packages(id),
    from_version_struct app.semver not null,
    from_version text not null generated always as (app.semver_to_text(from_version_struct)) stored,
    to_version_struct app.semver not null,
    to_version text not null generated always as (app.semver_to_text(to_version_struct)) stored,
    sql varchar(250000),
    created_at timestamptz not null default now(),
    unique(package_id, from_version_struct, to_version_struct)
);

create function app.version_text_to_handle(version text)
    returns app.valid_name
    immutable
    language sql
as $$
    select split_part($1, '-', 1)
$$;

create function app.version_text_to_package_partial_name(version text)
    returns app.valid_name
    immutable
    language sql
as $$
    select split_part($1, '--', 2)
$$;
