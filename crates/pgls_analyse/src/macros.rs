/// This macro is used to declare an analyser rule type, and implement the
//  [RuleMeta] trait for it
///  # Example
///
/// The macro itself expect the following syntax:
///
/// ```rust,ignore
///use pgls_analyse::declare_rule;
///
/// declare_lint_rule! {
///     /// Documentation
///     pub(crate) ExampleRule {
///         version: "1.0.0",
///         name: "rule-name",
///         recommended: false,
///     }
/// }
/// ```
///
/// Check [crate](module documentation) for a better
/// understanding of how the macro works
#[macro_export]
macro_rules! declare_lint_rule {
    ( $( #[doc = $doc:literal] )+ $vis:vis $id:ident {
        version: $version:literal,
        name: $name:tt,
        severity: $severity:expr_2021,
        $( $key:ident: $value:expr_2021, )*
    } ) => {

        pgls_analyse::declare_rule!(
            $( #[doc = $doc] )*
            $vis $id {
                version: $version,
                name: $name,
                severity: $severity,
                $( $key: $value, )*
            }
        );

        // Declare a new `rule_category!` macro in the module context that
        // expands to the category of this rule
        // This is implemented by calling the `group_category!` macro from the
        // parent module (that should be declared by a call to `declare_group!`)
        // and providing it with the name of this rule as a string literal token
        #[allow(unused_macros)]
        macro_rules! rule_category {
            () => { super::group_category!( $name ) };
        }
    };
}

#[macro_export]
macro_rules! declare_rule {
        ( $( #[doc = $doc:literal] )+ $vis:vis $id:ident {
        version: $version:literal,
        name: $name:tt,
        severity: $severity:expr_2021,
        $( $key:ident: $value:expr_2021, )*
    } ) => {
        $( #[doc = $doc] )*
        $vis enum $id {}

        impl $crate::RuleMeta for $id {
            type Group = super::Group;
            const METADATA: $crate::RuleMetadata =
                $crate::RuleMetadata::new($version, $name, concat!( $( $doc, "\n", )* ), $severity) $( .$key($value) )*;
        }
    }
}

/// This macro is used by the codegen script to declare an analyser rule group,
/// and implement the [RuleGroup] trait for it
#[macro_export]
macro_rules! declare_lint_group {
    ( $vis:vis $id:ident { name: $name:tt, rules: [ $( $( $rule:ident )::* , )* ] } ) => {
        $vis enum $id {}

        impl $crate::RuleGroup for $id {
            type Category = super::Category;

            const NAME: &'static str = $name;

            fn record_rules<V: $crate::RegistryVisitor + ?Sized>(registry: &mut V) {
                $( registry.record_rule::<$( $rule )::*>(); )*
            }
        }

        pub(self) use $id as Group;

        // Declare a `group_category!` macro in the context of this module (and
        // all its children). This macro takes the name of a rule as a string
        // literal token and expands to the category of the lint rule with this
        // name within this group.
        // This is implemented by calling the `category_concat!` macro with the
        // "lint" prefix, the name of this group, and the rule name argument
        #[allow(unused_macros)]
        macro_rules! group_category {
            ( $rule_name:tt ) => { $crate::category_concat!( "lint", $name, $rule_name ) };
        }

        // Re-export the macro for child modules, so `declare_rule!` can access
        // the category of its parent group by using the `super` module
        pub(self) use group_category;
    };
}

#[macro_export]
macro_rules! declare_category {
    ( $vis:vis $id:ident { kind: $kind:ident, groups: [ $( $( $group:ident )::* , )* ] } ) => {
        $vis enum $id {}

        impl $crate::GroupCategory for $id {
            const CATEGORY: $crate::RuleCategory = $crate::RuleCategory::$kind;

            fn record_groups<V: $crate::RegistryVisitor + ?Sized>(registry: &mut V) {
                $( registry.record_group::<$( $group )::*>(); )*
            }
        }

        pub(self) use $id as Category;
    };
}
