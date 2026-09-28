-- Default configuration: json/jsonb_build_object group key/value pairs without any
-- explicit functionArgumentGroups entry. The 'invoiceNumber' pair is long enough to
-- fit at width 100 but break at width 80, exercising the fill fallback inside a group.
SELECT jsonb_build_object(
    'amountTTC',
    to_amount(amount_ttc, invoices.currency),
    'amountVAT',
    to_amount(amount_vat, invoices.currency),
    'invoiceReference',
    format_invoice_number(invoices.series, invoices.issued_on),
    'id',
    users.id
),
json_build_object(
    'profile',
    jsonb_build_object('name', users.name, 'age', users.age)
)
FROM users;
