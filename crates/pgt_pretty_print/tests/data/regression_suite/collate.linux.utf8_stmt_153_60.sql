do $$
BEGIN
  EXECUTE 'CREATE COLLATION test0 (locale = ' ||
          quote_literal((SELECT datcollate FROM pg_database WHERE datname = current_database())) || ');';
END
$$;
