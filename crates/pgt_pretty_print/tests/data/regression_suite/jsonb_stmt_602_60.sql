SELECT count(*) from testjsonb  WHERE j->'array' ? 'bar';
