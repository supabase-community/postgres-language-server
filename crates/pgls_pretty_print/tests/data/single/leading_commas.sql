-- pgls-format: commaStyle=leading, indentStyle=tabs, indentSize=4, lineWidth=80
SELECT
	staging.buildings.identification_number,
	staging.buildings.construction_year,
	staging.buildings.address_fk
FROM staging.buildings;
