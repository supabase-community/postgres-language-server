do $$
BEGIN
  EXECUTE 'CREATE COLLATION test1 (provider = icu, locale = ' ||
          quote_literal((SELECT CASE WHEN datlocprovider='i' THEN datlocale ELSE datcollate END FROM pg_database WHERE datname = current_database())) || ');';
END
$$;
