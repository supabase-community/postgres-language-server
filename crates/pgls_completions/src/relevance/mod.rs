pub(crate) mod filtering;
pub(crate) mod scoring;

use crate::providers::SqlKeyword;

#[derive(Debug, Clone)]
pub(crate) enum CompletionRelevanceData<'a> {
    Table(&'a pgls_catalog::Table),
    Function(&'a pgls_catalog::Function),
    Column(&'a pgls_catalog::Column),
    Parameter {
        function: &'a pgls_resolve::FunctionContext,
        param: &'a pgls_resolve::FunctionParam,
    },
    Schema(&'a pgls_catalog::Schema),
    Policy(&'a pgls_catalog::Policy),
    Role(&'a pgls_catalog::Role),
    Keyword(&'static SqlKeyword),
}
