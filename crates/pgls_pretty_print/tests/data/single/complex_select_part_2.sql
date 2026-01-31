  select
    view_id, view_schema, view_name,
    -- the following formatting is without indentation on purpose
    -- to allow simple diffs, with less whitespace noise
    replace(
      replace(
      replace(
      replace(
      replace(
      replace(
      replace(
      regexp_replace(
      replace(
      replace(
      replace(
      replace(
      replace(
      replace(
      replace(
      replace(
      replace(
      replace(
      replace(
        view_definition::text,
      -- This conversion to json is heavily optimized for performance.
      -- The general idea is to use as few regexp_replace() calls as possible.
      -- Simple replace() is a lot faster, so we jump through some hoops
      -- to be able to use regexp_replace() only once.
      -- This has been tested against a huge schema with 250+ different views.
      -- The unit tests do NOT reflect all possible inputs. Be careful when changing this!
      -- -----------------------------------------------
      -- pattern           | replacement         | flags
      -- -----------------------------------------------
      -- <> in pg_node_tree is the same as null in JSON, but due to very poor performance of json_typeof
      -- we need to make this an empty array here to prevent json_array_elements from throwing an error
      -- when the targetList is null.
      -- We'll need to put it first, to make the node protection below work for node lists that start with
      -- null: (<> ..., too. This is the case for coldefexprs, when the first column does not have a default value.
         '<>'              , '()'
      -- , is not part of the pg_node_tree format, but used in the regex.
      -- This removes all , that might be part of column names.
      ), ','               , ''
      -- The same applies for { and }, although those are used a lot in pg_node_tree.
      -- We remove the escaped ones, which might be part of column names again.
      ), E'\\\\{'            , ''
      ), E'\\\\}'            , ''
      -- The fields we need are formatted as json manually to protect them from the regex.
      ), ' :targetList '   , ',"targetList":'
      ), ' :resno '        , ',"resno":'
      ), ' :resorigtbl '   , ',"resorigtbl":'
      ), ' :resorigcol '   , ',"resorigcol":'
      -- Make the regex also match the node type, e.g. \`{QUERY ...\`, to remove it in one pass.
      ), '{'               , '{ :'
      -- Protect node lists, which start with \`({\` or \`((\` from the greedy regex.
      -- The extra \`{\` is removed again later.
      ), '(('              , '{(('
      ), '({'              , '{({'
      -- This regex removes all unused fields to avoid the need to format all of them correctly.
      -- This leads to a smaller json result as well.
      -- Removal stops at \`,\` for used fields (see above) and \`}\` for the end of the current node.
      -- Nesting can't be parsed correctly with a regex, so we stop at \`{\` as well and
      -- add an empty key for the followig node.
      ), ' :[^}{,]+'       , ',"":'              , 'g'
      -- For performance, the regex also added those empty keys when hitting a \`,\` or \`}\`.
      -- Those are removed next.
      ), ',"":}'           , '}'
      ), ',"":,'           , ','
      -- This reverses the "node list protection" from above.
      ), '{('              , '('
      -- Every key above has been added with a \`,\` so far. The first key in an object doesn't need it.
      ), '{,'              , '{'
      -- pg_node_tree has \`()\` around lists, but JSON uses \`[]\`
      ), '('               , '['
      ), ')'               , ']'
      -- pg_node_tree has \` \` between list items, but JSON uses \`,\`
      ), ' '             , ','
    )::json as view_definition
  from views
