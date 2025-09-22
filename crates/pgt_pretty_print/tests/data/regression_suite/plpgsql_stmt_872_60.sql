do $$
declare v_test plpgsql_arr_domain := array[1];
begin
  v_test := 0 || v_test;  -- fail
end;
$$;
