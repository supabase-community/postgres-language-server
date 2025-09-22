DO $$ -- use DO to protect -- from psql
  declare r boolean;
  begin
    execute $e$ select 2 !=-- comment
      1 $e$ into r;
    raise info 'r = %', r;
  end;
$$;
