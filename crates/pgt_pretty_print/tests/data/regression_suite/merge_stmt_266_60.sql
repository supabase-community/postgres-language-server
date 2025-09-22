SELECT explain_merge('
MERGE INTO ex_mtarget t USING ex_msource s ON t.a = s.a
WHEN NOT MATCHED BY SOURCE and t.a < 10 THEN
	DELETE');
