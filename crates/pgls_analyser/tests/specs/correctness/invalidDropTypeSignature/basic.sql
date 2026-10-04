drop type if exists group_composite;
-- expect_lint/invalidDropTypeSignature
drop type if exists group_composite (int, text);
