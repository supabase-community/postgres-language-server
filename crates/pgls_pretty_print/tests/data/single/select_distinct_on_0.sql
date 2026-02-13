SELECT DISTINCT ON (department_id, team_id) employee_id, team_id FROM employees ORDER BY department_id, team_id;
