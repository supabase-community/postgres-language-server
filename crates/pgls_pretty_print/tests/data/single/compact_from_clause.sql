-- pgls-format: clauseBodyStyle=compact, indentStyle=tabs, indentSize=4, lineWidth=80
SELECT
	staging.buildings.id,
	staging.addresses.city
FROM staging.buildings
	LEFT JOIN staging.addresses ON staging.addresses.id = staging.buildings.address_fk
WHERE staging.buildings.construction_year > 1950;
