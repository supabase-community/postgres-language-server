-- expect_lint/safety/banCharField
CREATE TABLE "core_bar" (
    "id" serial NOT NULL PRIMARY KEY,
    "alpha" char(100) NOT NULL
);