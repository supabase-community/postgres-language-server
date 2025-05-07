pub(crate) mod filtering;
pub(crate) mod scoring;

#[derive(Debug, Clone)]
pub(crate) enum CompletionRelevanceData<'a> {
    Table(&'a pgt_schema_cache::Table),
    Function(&'a pgt_schema_cache::Function),
    Column(&'a pgt_schema_cache::Column),
    Schema(&'a pgt_schema_cache::Schema),
}
