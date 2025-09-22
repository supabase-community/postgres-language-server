CREATE UNIQUE INDEX parallel_hang_idx
					ON parallel_hang
					USING btree (i int4_custom_ops);
