-- pgls-format: layout=expanded, lineWidth=200, indentStyle=tabs, indentSize=4, keywordCase=upper, constantCase=upper, typeCase=upper, commaStyle=leading, logicalOperatorPlacement=leading
SELECT
    CASE
        WHEN accounting_lines.accounting_class::varchar SIMILAR TO '(6|7)%'
            AND accounting_lines.accounting_class::varchar NOT SIMILAR TO '70(1|2|4)%'
            THEN coalesce(accounting_lines.allocation_key_fk, allocation_keys.id)
        ELSE accounting_lines.allocation_key_fk
    END AS allocation_key_fk,
    CASE WHEN source_software = 'TTW' THEN 'THETRAWIN' WHEN target_tenant = 'esset' THEN 'ESSET' ELSE 'EXTERNAL_GROWTH' END AS source
    , CASE
        WHEN bank_informations.account_type = 'SEPARATE'
            AND account_nature IN ('COMPTE_COURANT', 'COMPTE_COURANT_OTHER')
            AND accounting_class = '5121'
            AND accounting_account_sub_account = '000000003'
            AND is_main_bank_account = TRUE
            AND buildings.id IS NOT NULL
            THEN concat('SDC ', buildings.name)
        WHEN kind = 'CO_OWNER'
            THEN coalesce(bank_informations.name, clean_labels(co_owner_account_customers.fullname))
        WHEN kind = 'TENANT'
            THEN coalesce(bank_informations.name, clean_labels(lease_customers.fullname))
        WHEN kind = 'LANDLORD'
            THEN coalesce(bank_informations.name, clean_labels(landlord_customers.fullname))
        ELSE bank_informations.name
    END AS account_name
FROM accounting_lines;
