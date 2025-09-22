create function return_int_input(int) returns int as $$
begin
	return $1;
end;
$$ language plpgsql stable;
