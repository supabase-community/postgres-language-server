//! Generated file, do not edit by hand, see `xtask/codegen`

use pgls_analyse::declare_lint_group;
pub(crate) mod ambiguous_column;
pub(crate) mod assignment_type_mismatch;
pub(crate) mod function_argument_mismatch;
pub(crate) mod function_return_type_mismatch;
pub(crate) mod generated_column_assignment;
pub(crate) mod insert_column_mismatch;
pub(crate) mod invalid_cast;
pub(crate) mod invalid_merge;
pub(crate) mod invalid_select_into;
pub(crate) mod missing_from_clause_entry;
pub(crate) mod operator_type_mismatch;
pub(crate) mod unknown_column;
pub(crate) mod unknown_function;
pub(crate) mod unknown_relation;
pub(crate) mod unknown_schema;
pub(crate) mod unknown_type;
declare_lint_group! { pub Typecheck { name : "typecheck" , rules : [self :: ambiguous_column :: AmbiguousColumn , self :: assignment_type_mismatch :: AssignmentTypeMismatch , self :: function_argument_mismatch :: FunctionArgumentMismatch , self :: function_return_type_mismatch :: FunctionReturnTypeMismatch , self :: generated_column_assignment :: GeneratedColumnAssignment , self :: insert_column_mismatch :: InsertColumnMismatch , self :: invalid_cast :: InvalidCast , self :: invalid_merge :: InvalidMerge , self :: invalid_select_into :: InvalidSelectInto , self :: missing_from_clause_entry :: MissingFromClauseEntry , self :: operator_type_mismatch :: OperatorTypeMismatch , self :: unknown_column :: UnknownColumn , self :: unknown_function :: UnknownFunction , self :: unknown_relation :: UnknownRelation , self :: unknown_schema :: UnknownSchema , self :: unknown_type :: UnknownType ,] } }
