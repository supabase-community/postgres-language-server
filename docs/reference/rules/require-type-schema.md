# requireTypeSchema

**Group** [`style`](../rules.md#style) · **Since** `Unreleased`  
**Diagnostic** `lint/requireTypeSchema`

## Description
Type names should be schema-qualified.

Without a schema, Postgres looks the type up in the schemas on the `search_path`. A type
with the same name in a schema earlier on the search path is used instead of the intended
one, which changes what a cast or a function call does, and which function overload is
called.

The rule is meant for code where this matters, like functions with `SECURITY DEFINER` and
extension scripts, as defence in depth next to setting the `search_path` explicitly.

The rule checks every type name, like the ones of casts, column definitions, and function
arguments and results. The types whose names are SQL keywords, like `integer`, `varchar`,
`timestamp` or `boolean`, always refer to `pg_catalog` and are not reported. Types like
`text`, `uuid` or `jsonb` are not keywords and are reported. The serial pseudo-types and
`%TYPE` references are not reported either.

## Examples

### Invalid

```sql
select 'hello'::text;
```

```sh
code-block.sql:1:17 lint/requireTypeSchema ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ! Type text is not schema-qualified.
  
  > 1 │ select 'hello'::text;
      │                 ^^^^
    2 │ 
  
  i Without a schema, the type is resolved through the search_path: a type with the same name in a schema earlier on the search_path is used instead.
  
  i Specify the schema, e.g. pg_catalog.text for a built-in type.
  

```

### Valid

```sql
select 'hello'::pg_catalog.text;
select 1::integer;
create table public.users (id serial, created_at timestamp with time zone);
```

## How to configure
```json

{
  "linter": {
    "rules": {
      "requireTypeSchema": "error"
    }
  }
}

```
## How to suppress

Suppress this diagnostic with a comment:

```sql
-- pgls-ignore requireTypeSchema
```
