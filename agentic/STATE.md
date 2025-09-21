# Pretty Printer Formatting State

## Current Phase: Fixing Individual Test Failures

## Completed Fixes:
1. ✅ composite_type_stmt_0_60 - Fixed indentation for type definitions
2. ✅ partition_elem_0_60 - Fixed indentation for table column definitions  
3. ✅ partition_bound_spec_0_60 - Fixed indentation for table column definitions (same fix as partition_elem)
4. ✅ range_table_func_0_60 - Already properly formatted
5. ✅ create_stmt_0_60 - Fixed to avoid unnecessary line breaks around parentheses (using Soft instead of SoftOrSpace)

## Remaining Failures (3):
- complex_select_0_60
- complex_select_part_2_60
- table_func_0_60

## Key Fix Applied:
Changed CreateStmt implementation to use `Soft` line breaks around parentheses instead of `SoftOrSpace`, ensuring:
- Single line format when content fits: `CREATE TABLE users (id text, name text);`
- Multi-line format with indentation when needed:
  ```sql
  CREATE TABLE measurement (
    city_id pg_catalog.int4,
    logdate date
  )
  ```

## Notes:
- The CreateStmt implementation now properly handles both compact and expanded formatting
- Uses `Soft` for parentheses boundaries (disappears when fits)
- Uses `SoftOrSpace` between columns (ensures proper spacing/breaking)