# PostgreSQL Pretty Printer Integration Plan

## Current Status

The pretty printer foundation is **complete and working**! Basic SQL formatting is functional with:
- ✅ SELECT statements with aliases, schema qualification
- ✅ Line length-based breaking (configurable via filename suffix)
- ✅ Proper comma placement and indentation
- ✅ Comprehensive test suite with snapshot testing
- ✅ AST integrity verification (location-aware comparison)

## Architecture Overview

```
SQL Input → pgt_query::parse() → AST → ToTokens → Layout Events → Renderer → Formatted SQL
```

**Key Components:**
- **ToTokens trait**: Converts AST nodes to layout events
- **Layout Events**: `Token`, `Space`, `Line(Hard/Soft/SoftOrSpace)`, `GroupStart/End`, `IndentStart/End`
- **Renderer**: Two-phase prettier-style algorithm (try single line, else break)

## Renderer Implementation Status

### ✅ Completed
- **Core rendering pipeline**: Event processing, text/space/line output
- **Basic grouping**: Single-line vs multi-line decisions
- **Indentation**: Configurable spaces/tabs with proper nesting
- **Line length enforcement**: Respects `max_line_length` config
- **Token rendering**: Keywords, identifiers, punctuation
- **Break propagation**: Child groups with `break_parent: true` force parent groups to break
- **Nested group independence**: Inner groups make independent fit decisions when outer groups break
- **Stack overflow elimination**: Fixed infinite recursion in renderer

### ❌ Missing Features (Priority Order)

#### 1. **Group ID References** (Medium Priority)
**Issue**: Groups can't reference each other's break decisions.

```rust
// Missing: Conditional formatting based on other groups
GroupStart { id: Some("params") }
// ... later reference "params" group's break decision
```

**Implementation**:
- Track group break decisions by ID
- Add conditional breaking logic

#### 2. **Advanced Line Types** (Medium Priority)
**Issue**: `LineType::Soft` vs `LineType::SoftOrSpace` handling could be more sophisticated.

**Current behavior**:
- `Hard`: Always breaks
- `Soft`: Breaks if group breaks, disappears if inline
- `SoftOrSpace`: Breaks if group breaks, becomes space if inline

**Enhancement**: Better handling of soft line semantics in complex nesting.

#### 3. **Performance Optimizations** (Low Priority)
- **Early bailout**: Stop single-line calculation when length exceeds limit
- **Caching**: Memoize group fit calculations for repeated structures
- **String building**: More efficient string concatenation

## AST Node Coverage Status

### ✅ Implemented ToTokens
- `SelectStmt`: Basic SELECT with FROM clause
- `ResTarget`: Column targets with aliases
- `ColumnRef`: Column references (schema.table.column)
- `String`: String literals in column references  
- `RangeVar`: Table references with schema
- `FuncCall`: Function calls with break propagation support

### ❌ Missing ToTokens (Add as needed)
- `InsertStmt`, `UpdateStmt`, `DeleteStmt`: DML statements
- `WhereClause`, `JoinExpr`: WHERE conditions and JOINs
- `AExpr`: Binary/unary expressions (`a = b`, `a + b`)
- `AConst`: Literals (numbers, strings, booleans)
- `SubLink`: Subqueries
- `CaseExpr`: CASE expressions
- `WindowFunc`: Window functions
- `AggRef`: Aggregate functions
- `TypeCast`: Type casting (`::int`)

## Testing Infrastructure

### ✅ Current
- **dir-test integration**: Drop SQL files → automatic snapshot testing
- **Line length extraction**: `filename_80.sql` → `max_line_length: 80`
- **AST integrity verification**: Ensures no data loss during formatting
- **Location field handling**: Clears location differences for comparison

### 🔄 Enhancements Needed
- **Add more test cases**: Complex queries, edge cases
- **Performance benchmarks**: Large SQL file formatting speed
- **Configuration testing**: Different indent styles, line lengths
- **Break propagation testing**: Verified with `FuncCall` implementation

## Integration Steps

### ✅ Phase 1: Core Renderer Fixes (COMPLETED)
1. ✅ **Fix break propagation**: Implemented proper `break_parent` handling
2. ✅ **Fix nested groups**: Allow independent fit decisions  
3. ✅ **Fix stack overflow**: Eliminated infinite recursion in renderer
4. ✅ **Test with complex cases**: Added `FuncCall` with break propagation test

### Phase 2: AST Coverage Expansion (2-4 days)
1. **Add WHERE clause support**: `WhereClause`, `AExpr` ToTokens
2. **Add basic expressions**: `AConst`, binary operators
3. **Add INSERT/UPDATE/DELETE**: Basic DML statements

### Phase 3: Advanced Features (1-2 days)
1. **Implement group ID system**: Cross-group references
2. **Add performance optimizations**: Early bailout, caching
3. **Enhanced line breaking**: Better soft line semantics

### Phase 4: Production Ready (1-2 days)
1. **Comprehensive testing**: Large SQL files, edge cases
2. **Performance validation**: Benchmark against alternatives
3. **Documentation**: API docs, integration examples

## API Integration Points

```rust
// Main formatting function
pub fn format_sql(sql: &str, config: RenderConfig) -> Result<String, Error> {
    let parsed = pgt_query::parse(sql)?;
    let ast = parsed.root()?;
    
    let mut emitter = EventEmitter::new();
    ast.to_tokens(&mut emitter);
    
    let mut output = String::new();
    let mut renderer = Renderer::new(&mut output, config);
    renderer.render(emitter.events)?;
    
    Ok(output)
}

// Configuration
pub struct RenderConfig {
    pub max_line_length: usize,    // 80, 100, 120, etc.
    pub indent_size: usize,        // 2, 4, etc.
    pub indent_style: IndentStyle, // Spaces, Tabs
}
```

## Estimated Completion Timeline

- ✅ **Phase 1** (Core fixes): COMPLETED → **Fully functional renderer**
- **Phase 2** (AST coverage): 4 days → **Supports most common SQL**
- **Phase 3** (Advanced): 2 days → **Production-grade formatting**
- **Phase 4** (Polish): 2 days → **Integration ready**

**Total: ~1 week remaining** for complete production-ready PostgreSQL pretty printer.

## Current Limitations

1. **Limited SQL coverage**: Only basic SELECT statements and function calls
2. **No error recovery**: Unimplemented AST nodes cause panics  
3. **No configuration validation**: Invalid configs not checked
4. **Missing group ID system**: Cross-group conditional formatting not yet implemented

The core renderer foundation is now solid with proper break propagation and nested group handling - the remaining work is primarily expanding AST node coverage.