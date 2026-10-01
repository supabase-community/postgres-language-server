//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub mod ambiguous_column;
pub mod assignment_type_mismatch;
pub mod function_argument_mismatch;
pub mod function_return_type_mismatch;
pub mod insert_column_mismatch;
pub mod invalid_cast;
pub mod missing_from_clause_entry;
pub mod operator_type_mismatch;
pub mod unknown_column;
pub mod unknown_function;
pub mod unknown_relation;
pub mod unknown_schema;
pub mod unknown_type;
declare_lint_group! { pub Typecheck { name : "typecheck" , rules : [self :: ambiguous_column :: AmbiguousColumn , self :: assignment_type_mismatch :: AssignmentTypeMismatch , self :: function_argument_mismatch :: FunctionArgumentMismatch , self :: function_return_type_mismatch :: FunctionReturnTypeMismatch , self :: insert_column_mismatch :: InsertColumnMismatch , self :: invalid_cast :: InvalidCast , self :: missing_from_clause_entry :: MissingFromClauseEntry , self :: operator_type_mismatch :: OperatorTypeMismatch , self :: unknown_column :: UnknownColumn , self :: unknown_function :: UnknownFunction , self :: unknown_relation :: UnknownRelation , self :: unknown_schema :: UnknownSchema , self :: unknown_type :: UnknownType ,] } }
