-- pgls-format: lineWidth=80, commaStyle=leading, functionArgumentGroups=my_pair_func:2
-- Leading commas apply to grouped arguments too: the break opportunity sits before the
-- comma, so a broken grouped list reads "\n, 'key', value".
SELECT my_pair_func(
    'amountTTC',
    to_amount(amount_ttc, invoices.currency),
    'amountVAT',
    to_amount(amount_vat, invoices.currency),
    'rateVAT',
    round(amount_ttc / amount_ht * 100)
) AS grouped,
jsonb_build_object(
    'id',
    users.id,
    'email',
    users.email,
    'profile',
    users.name
) AS ungrouped_default
FROM users;
