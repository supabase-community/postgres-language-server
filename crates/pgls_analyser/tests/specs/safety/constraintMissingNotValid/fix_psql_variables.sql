-- An identifier placeholder: the fix is checked with the placeholder read as an identifier.
-- expect_lint/constraintMissingNotValid
alter table :"schema".distributors add constraint distfk foreign key (address) references addresses (address);
