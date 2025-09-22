create rule qqq as on update to copydml_test where new.t <> 'f' do instead delete from copydml_test;
