-- pgls-format: isolateSemicolon=true, indentStyle=tabs, indentSize=4, lineWidth=80
SELECT
	staging.buildings.id,
	staging.buildings.construction_year,
	staging.buildings.address_fk
FROM staging.buildings;
