use globwalk::GlobWalkerBuilder;
use proc_macro::TokenStream;
use proc_macro_error::*;
use quote::*;
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

#[proc_macro]
#[proc_macro_error]
pub fn gen_tests(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as Arguments);

    match args.generate() {
        Ok(tokens) => tokens,
        Err(e) => abort!(e, "{}", e),
    }
}

/// A Recursive Tree Structure that stores tests per *part* of a Path.
///
/* foo
 * ├── bar
 * │   ├── testA.sql
 * ├── testB.sql
 *
 * Results in:
 *
 * TestModules {
 *     modules: {
 *        "foo": TestModules {
 *             modules: {
 *                 "bar": TestModules {
 *                     modules: {},
 *                     tests: [stream->testA.sql]
 *                  }
 *             }
 *             tests: [stream->testB.sql]
 *        }
 *     }
 *     tests: []
 * }
 *
 * Note that `tests` does not hold actual files but the TokenStreams for the tests for those files.
*/
#[derive(Default)]
struct TestModules {
    modules: HashMap<String, TestModules>,
    tests: Vec<proc_macro2::TokenStream>,
}

impl TestModules {
    fn insert<'a>(
        &mut self,
        mut path: impl Iterator<Item = &'a str>,
        test: proc_macro2::TokenStream,
    ) {
        match path.next() {
            Some(part) => {
                let module = self.modules.entry(part.into()).or_default();
                module.insert(path, test);
            }
            None => {
                self.tests.push(test);
            }
        }
    }

    fn print(self, output: &mut proc_macro2::TokenStream) {
        for (name, sub_module) in self.modules {
            let name = syn::Ident::new(&name, proc_macro2::Span::call_site());

            let mut sub_module_stream = proc_macro2::TokenStream::new();
            sub_module.print(&mut sub_module_stream);

            // wrap the submodule tests in a `mod`
            output.extend(quote! {
                    mod #name { #sub_module_stream }
            });
        }
        output.extend(self.tests)
    }
}

struct Arguments {
    pattern: syn::ExprLit,
    test_function: syn::Path,
}

impl syn::parse::Parse for Arguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern = input.parse()?;
        let _: syn::Token!(,) = input.parse()?;
        let test_function = input.parse()?;
        Ok(Arguments {
            pattern,
            test_function,
        })
    }
}

impl Arguments {
    fn get_filepaths(&self) -> Result<Vec<PathBuf>, &'static str> {
        let base = std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| "Cannot find CARGO_MANIFEST_DIR. Are you using cargo?")?;

        let pattern = match &self.pattern.lit {
            syn::Lit::Str(s) => s.value(),
            _ => return Err("Invalid pattern."),
        };

        let walker = GlobWalkerBuilder::new(base, pattern)
            .build()
            .map_err(|_| "Cannot build walker.")?;

        let mut paths = Vec::new();

        for entry in walker {
            let entry = entry.map_err(|_| "Error iteraring over entry.")?;

            let filename = entry
                .file_name()
                .to_str()
                .ok_or("Cannot convert filename to string.")?;

            if filename.ends_with(".expected.sql") {
                continue;
            }

            let meta = entry.metadata().map_err(|_| "Cannot open file.")?;

            if meta.is_file() {
                paths.push(entry.path().to_path_buf());
            }
        }

        Ok(paths)
    }

    fn generate(self) -> Result<TokenStream, &'static str> {
        let files = self.get_filepaths()?;
        let mut modules = TestModules::default();

        for file in files {
            let Variables {
                test_name,
                test_fullpath,
                test_expected_fullpath,
                test_dir,
            } = file.try_into()?;

            let path = Path::new(&test_fullpath)
                .parent()
                .expect("Do not put tests in root directory.")
                .components()
                .map(Component::as_os_str)
                .skip_while(|c| {
                    let bytes = c.as_encoded_bytes();
                    bytes != b"specs" && bytes != b"tests"
                })
                .filter_map(OsStr::to_str);

            let span = self.pattern.lit.span();
            let test_name = syn::Ident::new(&test_name, span);
            let func = &self.test_function;

            modules.insert(
                path,
                quote! {
                    #[test]
                    pub fn #test_name () {
                        let test_fullpath = #test_fullpath;
                        let test_expected_fullpath = #test_expected_fullpath;
                        let test_dir = #test_dir;
                        #func(test_fullpath, test_expected_fullpath, test_dir);
                    }
                },
            )
        }

        let mut output = proc_macro2::TokenStream::new();
        modules.print(&mut output);

        Ok(output.into())
    }
}

struct Variables {
    test_name: String,
    test_fullpath: String,
    test_expected_fullpath: String,
    test_dir: String,
}

impl TryFrom<PathBuf> for Variables {
    type Error = &'static str;

    fn try_from(mut path: PathBuf) -> Result<Self, Self::Error> {
        let test_name: String = path
            .file_stem()
            .ok_or("Cannot get file stem.")?
            .to_str()
            .ok_or("Cannot convert file stem to string.")?
            .into();

        let ext: String = path
            .extension()
            .ok_or("Cannot get extension.")?
            .to_str()
            .ok_or("Cannot convert extension to string.")?
            .into();
        assert_eq!(ext, "sql", "Expected .sql extension but received: {}", ext);

        let test_dir: String = path
            .parent()
            .ok_or("Cannot get parent directory.")?
            .to_str()
            .ok_or("Cannot convert parent directory to string.")?
            .into();

        let test_fullpath: String = path
            .as_os_str()
            .to_str()
            .ok_or("Cannot convert file stem to string.")?
            .into();

        path.set_extension(OsStr::new(""));

        let without_ext: String = path
            .as_os_str()
            .to_str()
            .ok_or("Cannot convert file stem to string.")?
            .into();

        let test_expected_fullpath = format!("{}.expected.{}", without_ext, ext);

        Ok(Variables {
            test_name,
            test_fullpath,
            test_expected_fullpath,
            test_dir,
        })
    }
}
