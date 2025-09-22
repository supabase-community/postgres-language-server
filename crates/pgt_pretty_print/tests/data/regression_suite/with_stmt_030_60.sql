WITH RECURSIVE subdepartment AS
(
	-- select all columns to prevent projection
	SELECT id, parent_department, name FROM department WHERE name = 'A'

	UNION

	-- joins do projection
	SELECT d.id, d.parent_department, d.name FROM department AS d
	INNER JOIN subdepartment AS sd ON d.parent_department = sd.id
)
SELECT * FROM subdepartment ORDER BY name;
