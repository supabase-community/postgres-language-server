insert into app.package_versions(package_id, version_struct, sql, description_md)
values (
(select id from app.packages where package_alias = 'olirice@index_advisor'),
(0,2,1),
$pkg$

-- Enforce requirements
-- Workaround to https://github.com/aws/pg_tle/issues/183
do $$
    declare
        hypopg_exists boolean = exists(
            select 1
            from pg_available_extensions
            where
                name = 'hypopg'
                and installed_version is not null
        );
    begin

        if not hypopg_exists then
            raise
                exception '"olirice@index_advisor" requires "hypopg"'
                using hint = 'Run "create extension hypopg" and try again';
        end if;
    end
$$;

create or replace function index_advisor(
    query text
)
    returns table  (
        startup_cost_before jsonb,
        startup_cost_after jsonb,
        total_cost_before jsonb,
        total_cost_after jsonb,
        index_statements text[],
        errors text[]
    )
    volatile
    language plpgsql
    as $$
declare
    n_args int;
    prepared_statement_name text = 'index_advisor_working_statement';
    hypopg_schema_name text = (select extnamespace::regnamespace::text from pg_extension where extname = 'hypopg');
    explain_plan_statement text;
    error_message text;
    rec record;
    plan_initial jsonb;
    plan_final jsonb;
    statements text[] = '{}';
begin

    -- Remove comment lines (its common that they contain semicolons)
    query := trim(
        regexp_replace(
            regexp_replace(
                regexp_replace(query,'\/\*.+\*\/', '', 'g'),
            '--[^\r\n]*', ' ', 'g'),
        '\s+', ' ', 'g')
    );

    -- Remove trailing semicolon
    query := regexp_replace(query, ';\s*$', '');

    begin
        -- Disallow multiple statements
        if query ilike '%;%' then
            raise exception 'Query must not contain a semicolon';
        end if;

        -- Hack to support PostgREST because the prepared statement for args incorrectly defaults to text
        query := replace(query, 'WITH pgrst_payload AS (SELECT $1 AS json_data)', 'WITH pgrst_payload AS (SELECT $1::json AS json_data)');

        -- Create a prepared statement for the given query
        deallocate all;
        execute format('prepare %I as %s', prepared_statement_name, query);

        -- Detect how many arguments are present in the prepared statement
        n_args = (
            select
                coalesce(array_length(parameter_types, 1), 0)
            from
                pg_prepared_statements
            where
                name = prepared_statement_name
            limit
                1
        );

        -- Create a SQL statement that can be executed to collect the explain plan
        explain_plan_statement = format(
            'set local plan_cache_mode = force_generic_plan; explain (format json) execute %I%s',
            --'explain (format json) execute %I%s',
            prepared_statement_name,
            case
                when n_args = 0 then ''
                else format(
                    '(%s)', array_to_string(array_fill('null'::text, array[n_args]), ',')
                )
            end
        );

        -- Store the query plan before any new indexes
        execute explain_plan_statement into plan_initial;

        -- Create possible indexes
        for rec in (
            with extension_regclass as (
                select
                    distinct objid as oid
                from
                    pg_catalog.pg_depend
                where
                    deptype = 'e'
            )
            select
                pc.relnamespace::regnamespace::text as schema_name,
                pc.relname as table_name,
                pa.attname as column_name,
                format(
                    'select %I.hypopg_create_index($i$create index on %I.%I(%I)$i$)',
                    hypopg_schema_name,
                    pc.relnamespace::regnamespace::text,
                    pc.relname,
                    pa.attname
                ) hypopg_statement
            from
                pg_catalog.pg_class pc
                join pg_catalog.pg_attribute pa
                    on pc.oid = pa.attrelid
                left join extension_regclass er
                    on pc.oid = er.oid
                left join pg_catalog.pg_index pi
                    on pc.oid = pi.indrelid
                    and (select array_agg(x) from unnest(pi.indkey) v(x)) = array[pa.attnum]
                    and pi.indexprs is null -- ignore expression indexes
                    and pi.indpred is null -- ignore partial indexes
            where
                pc.relnamespace::regnamespace::text not in ( -- ignore schema list
                    'pg_catalog', 'pg_toast', 'information_schema'
                )
                and er.oid is null -- ignore entities owned by extensions
                and pc.relkind in ('r', 'm') -- regular tables, and materialized views
                and pc.relpersistence = 'p' -- permanent tables (not unlogged or temporary)
                and pa.attnum > 0
                and not pa.attisdropped
                and pi.indrelid is null
                and pa.atttypid in (20,16,1082,1184,1114,701,23,21,700,1083,2950,1700,25,18,1042,1043)
            )
            loop
                -- Create the hypothetical index
                execute rec.hypopg_statement;
            end loop;

        -- Create a prepared statement for the given query
        -- The original prepared statement MUST be dropped because its plan is cached
        execute format('deallocate %I', prepared_statement_name);
        execute format('prepare %I as %s', prepared_statement_name, query);

        -- Store the query plan after new indexes
        execute explain_plan_statement into plan_final;

        --raise notice '%', plan_final;

        -- Idenfity referenced indexes in new plan
        execute format(
            'select
                coalesce(array_agg(hypopg_get_indexdef(indexrelid) order by indrelid, indkey::text), $i${}$i$::text[])
            from
                %I.hypopg()
            where
                %s ilike ($i$%%$i$ || indexname || $i$%%$i$)
            ',
            hypopg_schema_name,
            quote_literal(plan_final)::text
        ) into statements;

        -- Reset all hypothetical indexes
        perform hypopg_reset();

        -- Reset prepared statements
        deallocate all;

        return query values (
            (plan_initial -> 0 -> 'Plan' -> 'Startup Cost'),
            (plan_final -> 0 -> 'Plan' -> 'Startup Cost'),
            (plan_initial -> 0 -> 'Plan' -> 'Total Cost'),
            (plan_final -> 0 -> 'Plan' -> 'Total Cost'),
            statements::text[],
            array[]::text[]
        );
        return;

    exception when others then
        get stacked diagnostics error_message = MESSAGE_TEXT;

        return query values (
            null::jsonb,
            null::jsonb,
            null::jsonb,
            null::jsonb,
            array[]::text[],
            array[error_message]::text[]
        );
        return;
    end;

end;
$$;

$pkg$,

$description_md$

# index_advisor

`index_advisor` is an extension for recommending indexes to improve query performance.

## Installation

Note:

`hypopg` is a dependency of index_advisor.
Dependency resolution is currently under development.
In the future it will not be necessary to manually create dependencies.


```sql
select dbdev.install('olirice@index_advisor');
create extension if not exists hypopg;
create extension "olirice@index_advisor" version '0.2.0';
```

Features:
- Supports generic parameters e.g. `$1`, `$2`
- Supports materialized views
- Identifies tables/columns obfuscaed by views


## API

#### Description
For a given *query*, searches for a set of SQL DDL `create index` statements that improve the query's execution time;

#### Signature
```sql
index_advisor(query text)
returns
    table  (
        startup_cost_before jsonb,
        startup_cost_after jsonb,
        total_cost_before jsonb,
        total_cost_after jsonb,
        index_statements text[],
        errors text[]
    )
```

## Usage

For a minimal example, the `index_advisor` function can be given a single table query with a filter on an unindexed column.

```sql
create extension if not exists index_advisor cascade;

create table book(
  id int primary key,
  title text not null
);

select
    *
from
  index_advisor('select book.id from book where title = $1');

 startup_cost_before | startup_cost_after | total_cost_before | total_cost_after |                  index_statements                   | errors
---------------------+--------------------+-------------------+------------------+-----------------------------------------------------+--------
 0.00                | 1.17               | 25.88             | 6.40             | {"CREATE INDEX ON public.book USING btree (title)"},| {}
(1 row)
```

More complex queries may generate additional suggested indexes

```sql
create extension if not exists index_advisor cascade;

create table author(
    id serial primary key,
    name text not null
);

create table publisher(
    id serial primary key,
    name text not null,
    corporate_address text
);

create table book(
    id serial primary key,
    author_id int not null references author(id),
    publisher_id int not null references publisher(id),
    title text
);

create table review(
    id serial primary key,
    book_id int references book(id),
    body text not null
);

select
    *
from
    index_advisor('
        select
            book.id,
            book.title,
            publisher.name as publisher_name,
            author.name as author_name,
            review.body review_body
        from
            book
            join publisher
                on book.publisher_id = publisher.id
            join author
                on book.author_id = author.id
            join review
                on book.id = review.book_id
        where
            author.id = $1
            and publisher.id = $2
    ');

 startup_cost_before | startup_cost_after | total_cost_before | total_cost_after |                  index_statements                         | errors
---------------------+--------------------+-------------------+------------------+-----------------------------------------------------------+--------
 27.26               | 12.77              | 68.48             | 42.37            | {"CREATE INDEX ON public.book USING btree (author_id)",   | {}
                                                                                    "CREATE INDEX ON public.book USING btree (publisher_id)",
                                                                                    "CREATE INDEX ON public.review USING btree (book_id)"}
(3 rows)
```
$description_md$
);
