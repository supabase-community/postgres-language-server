-- expect_lint/safety/banAddExclusionConstraint
alter table my_table add constraint my_excl exclude using gist (col with &&);
