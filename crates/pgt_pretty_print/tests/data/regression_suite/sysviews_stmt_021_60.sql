select type, count(*) > 0 as ok FROM pg_wait_events
  where type <> 'InjectionPoint' group by type order by type COLLATE "C";
