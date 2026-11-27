use crate::{
    NodeEnum,
    protobuf::{CreateFunctionStmt, DefElem},
};

impl CreateFunctionStmt {
    /// The string value of an option, like `language`, or `as` with the body.
    ///
    /// The argument is a string or, for `as`, a list of strings. A list of two strings is the
    /// object file and the link symbol of a C function ([`interpret_AS_clause`]), so only a list
    /// of one string has a value.
    ///
    /// [`interpret_AS_clause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/functioncmds.c#L866
    pub fn option_value(&self, name: &str) -> Option<&str> {
        let option = DefElem::find(&self.options, name)?;
        match option.arg.as_deref()?.node.as_ref()? {
            NodeEnum::String(value) => Some(value.sval.as_str()),
            NodeEnum::List(list) => match list.items.as_slice() {
                [item] => item.string_value(),
                _ => None,
            },
            _ => None,
        }
    }

    /// [`Self::option_value`], to change the value. Like [`DefElem::find`], it reads the last
    /// option with the name; Postgres rejects a repeated option of `CREATE FUNCTION`.
    pub fn option_value_mut(&mut self, name: &str) -> Option<&mut String> {
        let option = DefElem::find_mut(&mut self.options, name)?;
        match option.arg.as_deref_mut()?.node.as_mut()? {
            NodeEnum::String(value) => Some(&mut value.sval),
            NodeEnum::List(list) => match list.items.as_mut_slice() {
                [item] => match item.node.as_mut()? {
                    NodeEnum::String(value) => Some(&mut value.sval),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{NodeEnum, protobuf::CreateFunctionStmt};

    fn create_function(sql: &str) -> CreateFunctionStmt {
        match crate::parse(sql).unwrap().into_root().unwrap() {
            NodeEnum::CreateFunctionStmt(stmt) => *stmt,
            _ => panic!("Expected CreateFunctionStmt"),
        }
    }

    #[test]
    fn option_values() {
        let create_fn = create_function(
            "
            CREATE OR REPLACE FUNCTION public.f1()
            RETURNS boolean
            LANGUAGE plpgsql
            AS $function$
            declare r t1 := (select t1 from t1 where a = 1);
            BEGIN
              if r.c is null or
                 true is false
                then -- there is bug - table t1 missing \"c\" column
                RAISE NOTICE 'c is null';
              end if;
            END;
            $function$;
"
            .trim(),
        );

        assert_eq!(create_fn.option_value("language"), Some("plpgsql"));
        assert!(create_fn.option_value("as").is_some());
    }

    #[test]
    fn the_object_file_of_a_c_function_is_not_its_body() {
        let create_fn =
            create_function("create function f() returns int language c as 'my_lib', 'my_symbol'");

        assert_eq!(create_fn.option_value("language"), Some("c"));
        assert_eq!(create_fn.option_value("as"), None);
    }

    #[test]
    fn changed_option_values() {
        let mut create_fn =
            create_function("create function f() returns int language sql as 'select 1'");

        *create_fn.option_value_mut("as").unwrap() = "select 2".to_owned();
        assert_eq!(create_fn.option_value("as"), Some("select 2"));
        assert_eq!(create_fn.option_value_mut("volatility"), None);
    }
}
