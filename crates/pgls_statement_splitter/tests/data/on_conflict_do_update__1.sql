INSERT INTO foo.bar (
    pk
) VALUES (
    $1
) ON CONFLICT (pk) DO UPDATE SET
    date_deleted = DEFAULT,
    date_created = DEFAULT;