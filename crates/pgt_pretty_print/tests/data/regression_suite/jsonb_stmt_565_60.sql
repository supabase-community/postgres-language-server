SELECT count(*) FROM testjsonb WHERE j @> '{"array":["bar"]}';
