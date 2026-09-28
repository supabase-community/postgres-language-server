-- pgls-format: lineWidth=80, functionArgumentGroups=jsonb_build_object:1|my_pair_func:2
-- `jsonb_build_object: 1` opts out of the built-in default grouping, while `my_pair_func: 2`
-- groups a custom function.
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
) AS ungrouped
FROM users;
