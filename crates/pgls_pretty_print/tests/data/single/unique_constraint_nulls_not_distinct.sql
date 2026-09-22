CREATE TABLE s.expense_types (
	expense_type_legacy_id TEXT,
	legacy_branch_code TEXT,
	UNIQUE NULLS NOT DISTINCT (expense_type_legacy_id, legacy_branch_code)
);
