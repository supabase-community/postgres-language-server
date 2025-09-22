create or replace function test_conv(
  input IN bytea,
  src_encoding IN text,
  dst_encoding IN text,

  result OUT bytea,
  errorat OUT bytea,
  error OUT text)
language plpgsql as
$$
declare
  validlen int;
begin
  -- First try to perform the conversion with noError = false. If that errors out,
  -- capture the error message, and try again with noError = true. The second call
  -- should succeed and return the position of the error, return that too.
  begin
    select * into validlen, result from test_enc_conversion(input, src_encoding, dst_encoding, false);
    errorat = NULL;
    error := NULL;
  exception when others then
    error := sqlerrm;
    select * into validlen, result from test_enc_conversion(input, src_encoding, dst_encoding, true);
    errorat = substr(input, validlen + 1);
  end;
  return;
end;
$$;
