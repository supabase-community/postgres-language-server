select
	jt.*
from
	jsonb_table_test jtt,
	json_table (
		jtt.js,'strict $[*]' as p
		columns (
			n for ordinality,
			a int path 'lax $.a' default -1 on empty,
			nested path 'strict $.b[*]' as pb columns (b_id for ordinality, b int path '$' ),
			nested path 'strict $.c[*]' as pc columns (c_id for ordinality, c int path '$' )
		)
	) jt;
