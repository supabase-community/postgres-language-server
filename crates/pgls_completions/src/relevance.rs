pub(crate) mod filtering;
pub(crate) mod scoring;

#[derive(Debug, Clone)]
pub(crate) enum CompletionRelevanceData<'a> {
    Table(&'a pgls_schema_cache::Table),
    Function(&'a pgls_schema_cache::Function),
    Column(&'a pgls_schema_cache::Column),
    Schema(&'a pgls_schema_cache::Schema),
    Policy(&'a pgls_schema_cache::Policy),
    Role(&'a pgls_schema_cache::Role),
}
