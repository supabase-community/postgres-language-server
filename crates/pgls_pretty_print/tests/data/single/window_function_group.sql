-- pgls-format: lineWidth=200, indentStyle=tabs, indentSize=4, keywordCase=upper, constantCase=upper, typeCase=upper, commaStyle=leading, logicalOperatorPlacement=leading, layout=expanded, castStyle=operator, clauseBodyStyle=break, isolateSemicolon=true
SELECT
	accounting_lines.id
	, COUNT(*) OVER (
		PARTITION BY accounting_lines.accounting_class
		, COALESCE(
			source.accounting_account_sub_account_source
			, accounting_lines.accounting_account_sub_account
		)
	) > 1 AS is_shared_target_account
FROM accounting_lines;
