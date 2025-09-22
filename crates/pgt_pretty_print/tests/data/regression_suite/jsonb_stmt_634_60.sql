SELECT count(*) FROM testjsonb WHERE j @@ 'exists($ ? (@.wait == null))';
