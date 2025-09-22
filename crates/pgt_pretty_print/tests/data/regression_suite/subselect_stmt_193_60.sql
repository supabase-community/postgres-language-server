create function bogus_int8_text_eq(int8, text) returns boolean
language sql as 'select $1::text = $2';
