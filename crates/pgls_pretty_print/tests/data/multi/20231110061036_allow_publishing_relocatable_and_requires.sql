-- A list of extensions which are allowed in the requires key of the control file
create table app.allowed_extensions (
    name text primary key
);

insert into app.allowed_extensions (name)
values
-- extensions available on Supabase
  ('citext'),
  ('pg_cron'),
  ('pg_graphql'),
  ('pg_stat_statements'),
  ('pg_trgm'),
  ('pg_crypto'),
  ('pg_jwt'),
  ('pg_sodium'),
  ('plpgsql'),
  ('uuid-ossp'),
  ('address_standardizer'),
  ('address_standardizer_data_us'),
  ('autoinc'),
  ('bloom'),
  ('btree_gin'),
  ('btree_gist'),
  ('cube'),
  ('dblink'),
  ('dict_int'),
  ('dict_xsyn'),
  ('earthdistance'),
  ('fuzzystrmatch'),
  ('hstore'),
  ('http'),
  ('hypopg'),
  ('insert_username'),
  ('intarray'),
  ('isn'),
  ('ltree'),
  ('moddatetime'),
  ('pg_hashids'),
  ('pg_jsonschema'),
  ('pg_net'),
  ('pg_repack'),
  ('pg_stat_monitor'),
  ('pg_walinspect'),
  ('pgaudit'),
  ('pgroonga'),
  ('pgroonga_database'),
  ('pgrouting'),
  ('pgrowlocks'),
  ('pgtap'),
  ('plcoffee'),
  ('pljava'),
  ('plls'),
  ('plpgsql_check'),
  ('plv8'),
  ('postgis'),
  ('postgis_raster'),
  ('postgis_sfcgal'),
  ('postgis_tiger_geocoder'),
  ('postgis_topology'),
  ('postgres_fdw'),
  ('refint'),
  ('rum'),
  ('seg'),
  ('sslinfo'),
  ('supautils'),
  ('tablefunc'),
  ('tcn'),
  ('timescaledb'),
  ('tsm_system_rows'),
  ('tsm_system_time'),
  ('unaccent'),
  ('vector'),
  ('wrappers'),

-- extensions available on AWS (except those already in Supabase)
-- full list here: https://docs.aws.amazon.com/AmazonRDS/latest/PostgreSQLReleaseNotes/postgresql-extensions.html
  ('amcheck'),
  ('aws_commons'),
  ('aws_lambda'),
  ('aws_s3'),
  ('bool_plperl'),
  ('decoder_raw'),
  ('h3-pg'),
  ('hll'),
  ('hstore_plperl'),
  ('intagg'),
  ('ip4r'),
  ('jsonb_plperl'),
  ('lo'),
  ('log_fdw'),
  ('mysql_fdw'),
  ('old_snapshot'),
  ('oracle_fdw'),
  ('orafce'),
  ('pageinspect'),
  ('pg_bigm'),
  ('pg_buffercache'),
  ('pg_freespacemap'),
  ('pg_hint_plan'),
  ('pg_partman'),
  ('pg_prewarm'),
  ('pg_proctab'),
  ('pg_similarity'),
  ('pg_tle'),
  ('pg_transport'),
  ('pg_visibility'),
  ('pgcrypto'),
  ('pgstattuple'),
  ('pgvector'),
  ('plperl'),
  ('plprofiler'),
  ('plrust'),
  ('pltcl'),
  ('prefix'),
  ('rdkit'),
  ('rds_tools'),
  ('tds_fdw'),
  ('test_parser'),
  ('wal2json');

grant insert (partial_name, handle, control_description, control_relocatable, control_requires)
    on app.packages
    to authenticated;

grant update (control_description, control_relocatable, control_requires)
    on app.packages
    to authenticated;

create or replace function public.publish_package(
    package_name app.valid_name,
    package_description varchar(1000),
    relocatable bool default false,
    requires text[] default '{}'
)
    returns void
    language plpgsql
as $$
declare
    account app.accounts = account from app.accounts account where id = auth.uid();
    require text;
begin
    if account.handle is null then
        raise exception 'user not logged in';
    end if;

    foreach require in array requires
    loop
        if not exists (
            select true
            from app.allowed_extensions
            where
                name = require
        ) then
            raise exception '`requires` in the control file can''t have `%` in it', require;
        end if;
    end loop;

    insert into app.packages(handle, partial_name, control_description, control_relocatable, control_requires)
    values (account.handle, package_name, package_description, relocatable, requires)
    on conflict on constraint packages_handle_partial_name_key
    do update
    set control_description = excluded.control_description,
        control_relocatable = excluded.control_relocatable,
        control_requires = excluded.control_requires;
end;
$$;
