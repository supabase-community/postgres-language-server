SELECT count(*) FROM testjsonb WHERE j @? '$ ? (@.wait == "CC" && true == @.public)';
