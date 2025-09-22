do $$
  declare r integer;
  begin
    select dfunc(a=>-- comment
      1) into r;
    raise info 'r = %', r;
  end;
$$;
