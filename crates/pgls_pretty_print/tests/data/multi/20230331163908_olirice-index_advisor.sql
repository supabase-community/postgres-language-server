insert into app.packages(
    handle,
    partial_name,
    control_description,
    control_relocatable,
    control_requires
)
values (
    'olirice',
    'index_advisor',
    'Recommend indexes for a given SQL query',
    true,
    '{hypopg}'
);


insert into app.package_versions(package_id, version_struct, sql, description_md)
values (
(select id from app.packages where package_name = 'olirice-index_advisor'),
(0,1,0),
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
                exception '"olirice-index_advisor" requires "hypopg"'
                using hint = 'Run "create extension hypopg" and try again';
        end if;
    end
$$;


create type index_advisor_output as (
    index_statements text[],
    startup_cost_before jsonb,
    startup_cost_after jsonb,
    total_cost_before jsonb,
    total_cost_after jsonb
);

create function index_advisor(
    query text
)
    returns table  (
        startup_cost_before jsonb,
        startup_cost_after jsonb,
        total_cost_before jsonb,
        total_cost_after jsonb,
        index_statements text[]
    )
    volatile
    language plpgsql
    as $$
declare
    n_args int;
    prepared_statement_name text = 'index_advisor_working_statement';
    hypopg_schema_name text = (select extnamespace::regnamespace::text from pg_extension where extname = 'hypopg');
    explain_plan_statement text;
    rec record;
    plan_initial jsonb;
    plan_final jsonb;
    statements text[] = '{}';
begin

    -- Disallow multiple statements
    if query ilike '%;%' then
        raise exception 'query must not contain a semicolon';
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
                pg_depend
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
            left join pg_index pi
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
        statements::text[]
    );

end;
$$;

$pkg$,

$description_md$

# index_advisor

`index_advisor` is an extension that recommends indexes to improve performance of a given query.

## Installation

Note:

`hypopg` is a dependency of index_advisor.
Dependency resolution is currently under development.
In the near future it will not be necessary to manually create dependencies.


```sql
select dbdev.install('olirice-index_advisor');
create extension if not exists hypopg;
create extension "olirice-index_advisor" cascade;
```

## Example

For a simple example, consider the following table:

```sql
create table book(
  id int primary key,
  title text not null
);
```

Lets say we want to query `book` by `title`, and return the relevant `id`.
That query would be `select book.id from book where title = $1`.

We can get `index_advisor` to recommend indexes that would improve performance on that query as follows:


```sql

select
    *
from
  index_advisor('select book.id from book where title = $1');

 startup_cost_before | startup_cost_after | total_cost_before | total_cost_after |                  index_statements
---------------------+--------------------+-------------------+------------------+-----------------------------------------------------
 0.00                | 1.17               | 25.88             | 6.40             | {"CREATE INDEX ON public.book USING btree (title)"},
(1 row)
```

where the output columns show top level statistics from the query explain plan (startup_cost, total_cost) and an array of `index_statements` that improve `total_cost`.

## Features:

- Generic parameters e.g. `$1`, `$2`
- Support for Materialized Views
- Identifies Tables/Columns Oobfuscaed by Views

## Usage

`index_advisor` is not limited to simple use cases. A more complex example could be:

```sql
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

 startup_cost_before | startup_cost_after | total_cost_before | total_cost_after |                  index_statements
---------------------+--------------------+-------------------+------------------+----------------------------------------------------------
 27.26               | 12.77              | 68.48             | 42.37            | {"CREATE INDEX ON public.book USING btree (author_id)",
                                                                                    "CREATE INDEX ON public.book USING btree (publisher_id)",
                                                                                    "CREATE INDEX ON public.review USING btree (book_id)"}
(1 row)
```

Note: the referenced tables must exist.

## API

```sql
index_advisor(query text)
returns
    table  (
        startup_cost_before jsonb,
        startup_cost_after jsonb,
        total_cost_before jsonb,
        total_cost_after jsonb,
        index_statements text[]
    )
```

#### Description
For a given *query*, searches for a set of SQL DDL `create index` statements that improve the query's execution time;

$description_md$

);
