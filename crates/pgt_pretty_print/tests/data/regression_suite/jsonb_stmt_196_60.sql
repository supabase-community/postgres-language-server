SELECT count(*) from testjsonb  WHERE j->'array' ? '5'::text;
