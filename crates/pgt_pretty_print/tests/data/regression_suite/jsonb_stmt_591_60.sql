SELECT count(*) FROM testjsonb WHERE j @? '$.wait ? ("CC" == @)';
