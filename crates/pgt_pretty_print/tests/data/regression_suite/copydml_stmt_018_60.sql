create rule qqq as on insert to copydml_test do instead (delete from copydml_test; delete from copydml_test);
