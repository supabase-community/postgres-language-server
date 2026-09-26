-- pgls-format: clauseBodyStyle=compact, indentStyle=tabs, indentSize=4, lineWidth=80
SELECT staging.buildings.id
FROM staging.buildings
WHERE staging.buildings.construction_year > 1950
	AND staging.buildings.address_fk IS NOT NULL;
