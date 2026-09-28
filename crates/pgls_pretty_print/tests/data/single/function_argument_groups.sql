-- pgls-format: lineWidth=80, functionArgumentGroups=jsonb_build_object:2
SELECT jsonb_build_object(
    'amountTTC',
    to_amount(amount_ttc, invoices.currency),
    'amountVAT',
    to_amount(amount_vat, invoices.currency),
    'rateVAT',
    CASE
        WHEN amount_ht = 0 THEN 0
        ELSE round(amount_ttc / amount_ht * 100)::INT
    END
);
