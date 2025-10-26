use insta::assert_snapshot;

fn printed_tree(sql: &str) -> String {
    let mut result = String::new();
    let mut parser = tree_sitter::Parser::new();
    let _ = parser.set_language(&pgt_treesitter_grammar::LANGUAGE.into());
    let tree = parser.parse(sql.trim(), None).expect("Unable to parse!");
    pgt_test_utils::print_ts_tree(&tree.root_node(), sql, 0, &mut result);
    result
}

#[test]
fn test_1() {
    let sql = "select * from auth.users;";

    assert_snapshot!(printed_tree(sql), @r"
    program [0..25] 'select * from auth.users;'
      statement [0..24] 'select * from auth.users'
        select [0..8] 'select *'
          keyword_select [0..6] 'select'
          select_expression [7..8] '*'
            term [7..8] '*'
              all_fields [7..8] '*'
                * [7..8] '*'
        from [9..24] 'from auth.users'
          keyword_from [9..13] 'from'
          relation [14..24] 'auth.users'
            object_reference [14..24] 'auth.users'
              any_identifier [14..18] 'auth'
              . [18..19] '.'
              any_identifier [19..24] 'users'
      ; [24..25] ';'
    ");
}

#[test]
fn test_2() {
    let sql1 = "update auth.users set email = 'my@mail.com';";
    let sql2 = "update auth.users set users.email = 'my@mail.com';";
    let sql3 = "update auth.users set auth.users.email = 'my@mail.com';";

    assert_snapshot!(printed_tree(sql1), @r"
    program [0..44] 'update auth.users set email = 'my@mail.com';'
      statement [0..43] 'update auth.users set email = 'my@mail.com''
        update [0..43] 'update auth.users set email = 'my@mail.com''
          keyword_update [0..6] 'update'
          relation [7..17] 'auth.users'
            object_reference [7..17] 'auth.users'
              any_identifier [7..11] 'auth'
              . [11..12] '.'
              any_identifier [12..17] 'users'
          keyword_set [18..21] 'set'
          assignment [22..43] 'email = 'my@mail.com''
            column_reference [22..27] 'email'
              column_identifier [22..27] 'email'
            = [28..29] '='
            literal [30..43] ''my@mail.com''
      ; [43..44] ';'
    ");

    assert_snapshot!(printed_tree(sql2), @r"
    program [0..50] 'update auth.users set users.email = 'my@mail.com';'
      statement [0..49] 'update auth.users set users.email = 'my@mail.com''
        update [0..49] 'update auth.users set users.email = 'my@mail.com''
          keyword_update [0..6] 'update'
          relation [7..17] 'auth.users'
            object_reference [7..17] 'auth.users'
              any_identifier [7..11] 'auth'
              . [11..12] '.'
              any_identifier [12..17] 'users'
          keyword_set [18..21] 'set'
          assignment [22..49] 'users.email = 'my@mail.com''
            column_reference [22..33] 'users.email'
              table_identifier [22..27] 'users'
              . [27..28] '.'
              column_identifier [28..33] 'email'
            = [34..35] '='
            literal [36..49] ''my@mail.com''
      ; [49..50] ';'
    ");

    assert_snapshot!(printed_tree(sql3), @r"
    program [0..55] 'update auth.users set auth.users.email = 'my@mail.com';'
      statement [0..54] 'update auth.users set auth.users.email = 'my@mail.com''
        update [0..54] 'update auth.users set auth.users.email = 'my@mail.com''
          keyword_update [0..6] 'update'
          relation [7..17] 'auth.users'
            object_reference [7..17] 'auth.users'
              any_identifier [7..11] 'auth'
              . [11..12] '.'
              any_identifier [12..17] 'users'
          keyword_set [18..21] 'set'
          assignment [22..54] 'auth.users.email = 'my@mail.com''
            column_reference [22..38] 'auth.users.email'
              schema_identifier [22..26] 'auth'
              . [26..27] '.'
              table_identifier [27..32] 'users'
              . [32..33] '.'
              column_identifier [33..38] 'email'
            = [39..40] '='
            literal [41..54] ''my@mail.com''
      ; [54..55] ';'
    ");
}
