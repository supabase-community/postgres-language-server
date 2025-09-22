create function list_part_fn(int) returns int as $$ begin return $1; end;$$ language plpgsql stable;
