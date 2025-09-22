create function parallel_safe_volatile(a int) returns int as
  $$ begin return a; end; $$ parallel safe volatile language plpgsql;
