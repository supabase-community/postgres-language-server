do $$
declare v_test plpgsql_domain := 1;
begin
  v_test := 0;  -- fail
end;
$$;
