create function returns_rw_array(int) returns int[]
language plpgsql as $$
  declare r int[];
  begin r := array[$1, $1]; return r; end;
$$ stable;
