create rule qqq as on insert to copydml_test where new.t <> 'f' do instead delete from copydml_test;
