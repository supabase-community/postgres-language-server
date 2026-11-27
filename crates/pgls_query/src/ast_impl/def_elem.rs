use crate::{
    NodeEnum,
    protobuf::{DefElem, Node},
};

impl DefElem {
    /// The option named `name` in a list of options, the last one if there are several.
    ///
    /// Postgres compares option names with `strcmp`, like `strcmp(opt->defname, "full")` in
    /// [`ExecVacuum`]. The parser lowercases unquoted names, so `VACUUM (FULL)` has the option
    /// `full`, while `VACUUM ("FULL")` has `FULL`, which Postgres rejects as unknown. `VACUUM` and
    /// `REINDEX` accept an option more than once, and the last one wins; most other statements
    /// reject it with [`errorConflictingDefElem`].
    ///
    /// [`ExecVacuum`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/vacuum.c#L243
    /// [`errorConflictingDefElem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L371
    pub fn find<'a>(options: &'a [Node], name: &str) -> Option<&'a DefElem> {
        options
            .iter()
            .rev()
            .find_map(|option| match option.node.as_ref()? {
                NodeEnum::DefElem(def) if def.defname == name => Some(def.as_ref()),
                _ => None,
            })
    }

    /// [`Self::find`], to change the option.
    pub fn find_mut<'a>(options: &'a mut [Node], name: &str) -> Option<&'a mut DefElem> {
        options
            .iter_mut()
            .rev()
            .find_map(|option| match option.node.as_mut()? {
                NodeEnum::DefElem(def) if def.defname == name => Some(def.as_mut()),
                _ => None,
            })
    }
}
