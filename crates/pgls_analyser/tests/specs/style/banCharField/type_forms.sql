-- The quoted "char" is a single-byte internal type, and myschema.bpchar a type of the user.
-- expect_lint/banCharField
-- expect_lint/banCharField
-- expect_lint/banCharField
-- expect_lint/banCharField
create table codes (
    a char(10)[],
    b CHARACTER(5) ARRAY,
    c national character(3),
    d char /* code */ (2),
    e "char",
    f myschema.bpchar
);
