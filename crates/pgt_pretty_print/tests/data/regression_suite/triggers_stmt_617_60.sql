create function trig_nothing() returns trigger language plpgsql
  as $$ begin return null; end $$;
