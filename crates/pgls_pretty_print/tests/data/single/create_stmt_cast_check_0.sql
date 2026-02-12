CREATE TABLE t (
    "type" "public"."my_enum" NOT NULL,
    "target_column" "text",
    "from_value" "text",
    "to_value" "text",
    CONSTRAINT "c2" CHECK (
        ("type" <> 'es_value_changed'::"public"."my_enum") OR
        (
            ("target_column" = 'config'::"text") AND
            (("from_value" IS NULL) AND ("to_value" IS NULL))
        ) OR
        (
            (("from_value" IS NOT NULL) AND ("to_value" IS NOT NULL)) AND
            ("target_column" IN ('type', 'name'))
        )
    )
);
