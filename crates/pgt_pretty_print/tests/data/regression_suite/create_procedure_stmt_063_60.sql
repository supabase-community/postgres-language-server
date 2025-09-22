CREATE PROCEDURE ptest11(a OUT int, VARIADIC b int[]) LANGUAGE SQL
  AS $$ SELECT b[1] + b[2] $$;
