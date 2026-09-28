-- Default configuration: json/jsonb_build_object group key/value pairs without any
-- explicit functionArgumentGroups entry.
SELECT jsonb_build_object(
    'amountTTC',
    to_amount(amount_ttc, invoices.currency),
    'amountVAT',
    to_amount(amount_vat, invoices.currency),
    'id',
    users.id,
    'email',
    users.email
),
json_build_object(
    'profile',
    jsonb_build_object('name', users.name, 'age', users.age)
)
FROM users;
