CREATE TABLE dropped_objects (
	object_type text,
	schema_name text,
	object_name text,
	object_identity text,
	address_names text[],
	address_args text[],
	is_temporary bool,
	original bool,
	normal bool
);
