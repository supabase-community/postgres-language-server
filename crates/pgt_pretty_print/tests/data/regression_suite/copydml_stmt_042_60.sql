create rule qqq as on delete to copydml_test do instead (insert into copydml_test default values; insert into copydml_test default values);
