insert into app.packages(
    handle,
    partial_name,
    control_description,
    control_relocatable,
    control_requires
)
values (
    'langchain',
    'hybrid_search',
    'Search documents by embedding and full text',
    true,
    '{pg_vector}'
);




insert into app.package_versions(package_id, version_struct, sql, description_md)
values (
(select id from app.packages where package_name = 'langchain-hybrid_search'),
(1,0,0),
$pkg$
-- Enforce requirements
-- Workaround to https://github.com/aws/pg_tle/issues/183
do $$
    declare
        dependencies_exists boolean = exists(
            select 1
            from pg_available_extensions
            where
                name = 'vector'
                and installed_version is not null
        );
    begin

        if not dependencies_exists then
            raise
                exception '"langchain-hybrid_search" requires "vector"'
                using hint = 'Run "create extension vector" and try again';
        end if;
    end
$$;

-- Create a table to store your documents
create table documents (
  id bigserial primary key,
  content text, -- corresponds to Document.pageContent
  metadata jsonb, -- corresponds to Document.metadata
  embedding vector(1536) -- 1536 works for OpenAI embeddings, change if needed
);

-- Create a function to similarity search for documents
create function match_documents (
  query_embedding vector(1536),
  match_count int
) returns table (
  id bigint,
  content text,
  metadata jsonb,
  similarity float
)
language plpgsql
as $$
#variable_conflict use_column
begin
  return query
  select
    id,
    content,
    metadata,
    1 - (documents.embedding <=> query_embedding) as similarity
  from documents
  order by documents.embedding <=> query_embedding
  limit match_count;
end;
$$;

-- Create a function to keyword search for documents
create function kw_match_documents(query_text text, match_count int)
returns table (id bigint, content text, metadata jsonb, similarity real)
as $$

begin
return query execute
format('
    select
        id, content, metadata, ts_rank(to_tsvector(content), plainto_tsquery($1)) as similarity
    from
        documents
    where
        to_tsvector(content) @@ plainto_tsquery($1)
    order by
        similarity desc
    limit $2
')
using query_text, match_count;
end;
$$ language plpgsql;

$pkg$,

$description_md$
# hybrid_search

Langchain supports hybrid search with a Supabase Postgres database. The hybrid search combines the postgres pgvector extension (similarity search) and Full-Text Search (keyword search) to retrieve documents. You can add documents via SupabaseVectorStore addDocuments function. SupabaseHybridKeyWordSearch accepts embedding, supabase client, number of results for similarity search, and number of results for keyword search as parameters. The getRelevantDocuments function produces a list of documents that has duplicates removed and is sorted by relevance score.

## Installation

```sql
select dbdev.install('langchain-hybrid_search');
create extension if not exists vector;
create extension "langchain-hybrid_search"
    schema public
    version '1.0.0';
```
Note:

`vector` is a dependency of `langchain-hybrid_search`.
Dependency resolution is currently under development.
In the near future it will not be necessary to manually create dependencies.


Once created, you can access the vector store for search using langchain as shown below:

```js
import { OpenAIEmbeddings } from "langchain/embeddings/openai";
import { createClient } from "@supabase/supabase-js";
import { SupabaseHybridSearch } from "langchain/retrievers/supabase";

const privateKey = process.env.SUPABASE_PRIVATE_KEY;
if (!privateKey) throw new Error(`Expected env var SUPABASE_PRIVATE_KEY`);

const url = process.env.SUPABASE_URL;
if (!url) throw new Error(`Expected env var SUPABASE_URL`);

export const run = async () => {
  const client = createClient(url, privateKey);

  const embeddings = new OpenAIEmbeddings();

  const retriever = new SupabaseHybridSearch(embeddings, {
    client,
    //  Below are the defaults, expecting that you set up your supabase table and functions according to the guide above. Please change if necessary.
    similarityK: 2,
    keywordK: 2,
    tableName: "documents",
    similarityQueryName: "match_documents",
    keywordQueryName: "kw_match_documents",
  });

  const results = await retriever.getRelevantDocuments("hello bye");

  console.log(results);
};
```

For more details, checkout the LangChain Supabase Hybrid Search docs: https://js.langchain.com/docs/modules/indexes/retrievers/supabase-hybrid
$description_md$
);
