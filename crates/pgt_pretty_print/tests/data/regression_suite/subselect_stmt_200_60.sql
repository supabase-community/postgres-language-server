create or replace function bogus_int8_text_eq(int8, text) returns boolean
language sql as 'select $2 = $1::text';
