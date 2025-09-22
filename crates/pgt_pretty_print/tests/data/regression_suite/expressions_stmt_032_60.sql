create function return_text_input(text) returns text as $$
begin
	return $1;
end;
$$ language plpgsql stable;
