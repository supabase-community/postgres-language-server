-- https://semver.org/#backusnaur-form-grammar-for-valid-semver-versions
create type app.semver_struct as (
    major smallint,
    minor smallint,
    patch smallint
);

create or replace function app.is_valid(app.semver_struct)
    returns boolean
    immutable
    language sql
as $$
    select (
        ($1).major is not null
        and ($1).minor is not null
        and ($1).patch is not null
    )
$$;

create domain app.semver
    as app.semver_struct
    check (
        app.is_valid(value)
);

create function app.semver_exception(version text)
    returns app.semver_struct
    immutable
    language plpgsql
as $$
begin
    raise exception using errcode='22000', message=format('Invalid semver %L', version);
end;
$$;


-- Cast from Text
create function app.text_to_semver(text)
    returns app.semver_struct
    immutable
    strict
    language sql
as $$
    with s(version) as (
        select (
            split_part($1, '.', 1),
            split_part($1, '.', 2),
            split_part(split_part(split_part($1, '.', 3), '-', 1), '+', 1)
        )::app.semver_struct
    )
    select
        case app.is_valid(s.version)
            when true then s.version
            else app.semver_exception($1)
       end
    from
        s
$$;


create or replace function app.semver_to_text(app.semver)
    returns text
    immutable
    language sql
as $$
    select
        format('%s.%s.%s', $1.major, $1.minor, $1.patch)
$$;
