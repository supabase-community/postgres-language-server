insert into src
  select string_agg(random()::text,'') from generate_series(1,10000);
