-- myschema.json is a type of the user.
-- expect_no_diagnostics
create table documents (
    body myschema.json
);
