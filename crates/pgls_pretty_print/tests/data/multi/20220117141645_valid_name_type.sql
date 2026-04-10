create extension if not exists citext with schema extensions;

create domain app.valid_name
    as extensions.citext
    check (
        -- 3 to 15 chars, A-z with underscores
        value ~ '^[A-z][A-z0-9\_]{2,32}$'
);

create or replace function app.exception(message text)
    returns text
        language plpgsql
        as $$
        begin
                raise exception using errcode='22000', message=message;
        end;
        $$;

/*
create domain app.valid_name
    as extensions.citext
    check (
        -- 3 to 15 chars, A-z with underscores
        case
            when value ~ '^[A-z][A-z0-9\_]{2,14}$' then True
            else app.exception('Bad name ' || value)::bool
        end
);
*/
