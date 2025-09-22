SELECT count(*) FROM testjsonb WHERE j @@ 'exists($ ? (@.array[*] == "bar"))';
