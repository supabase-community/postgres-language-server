SELECT count(*) FROM testjsonb WHERE j @@ 'exists($.public) || exists($.disabled)';
