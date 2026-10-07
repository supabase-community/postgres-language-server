-- pgls-format: lineWidth=200, indentStyle=tabs, indentSize=4, keywordCase=upper, commaStyle=leading, clauseBodyStyle=break, isolateSemicolon=true, layout=expanded
SELECT * FROM t LEFT OUTER JOIN payment_types AS source_payment_types ON source_payment_types.value_en = COALESCE(rent_issuance_parameters_payment_types.value, translating.leases.rent_issuance_parameters_payment_type);
