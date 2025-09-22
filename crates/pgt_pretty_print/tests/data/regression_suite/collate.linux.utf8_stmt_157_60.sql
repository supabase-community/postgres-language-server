do $$
BEGIN
  EXECUTE 'CREATE COLLATION test1 (lc_collate = ' ||
          quote_literal((SELECT datcollate FROM pg_database WHERE datname = current_database())) ||
          ', lc_ctype = ' ||
          quote_literal((SELECT datctype FROM pg_database WHERE datname = current_database())) || ');';
END
$$;
