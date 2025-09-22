create function stable_one() returns int as $$ begin return 1; end; $$ language plpgsql stable;
