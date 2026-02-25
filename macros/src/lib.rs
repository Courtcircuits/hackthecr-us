use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{LitStr, parse_macro_input};

fn to_pascal(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

/// Reads a JSON file at compile time and generates a `CrousRegion` enum whose
/// variants are the PascalCase keys of the JSON object, plus:
/// - `impl Display` (prints the original key name)
/// - `impl FromStr` (parses the original key name, case-sensitive)
/// - `fn url(&self) -> &'static str` returning the corresponding URL
///
/// Usage: `generate_crous_enum!("src/data/crous.json");`
/// The path is relative to the crate's `CARGO_MANIFEST_DIR`.
#[proc_macro]
pub fn generate_crous_enum(input: TokenStream) -> TokenStream {
    let path_lit = parse_macro_input!(input as LitStr);
    let relative_path = path_lit.value();

    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let full_path = std::path::Path::new(&manifest_dir).join(&relative_path);

    let content = std::fs::read_to_string(&full_path).unwrap_or_else(|e| {
        panic!("generate_crous_enum: failed to read {}: {}", full_path.display(), e)
    });

    let json: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|e| {
        panic!("generate_crous_enum: failed to parse JSON at {}: {}", full_path.display(), e)
    });

    let obj = json.as_object().unwrap_or_else(|| {
        panic!("generate_crous_enum: JSON root at {} must be an object", full_path.display())
    });

    let variants: Vec<_> = obj
        .keys()
        .map(|k| syn::Ident::new(&to_pascal(k), Span::call_site()))
        .collect();

    let keys: Vec<&str> = obj.keys().map(String::as_str).collect();

    let urls: Vec<&str> = obj
        .values()
        .map(|v| v.as_str().unwrap_or_default())
        .collect();

    let path_str = full_path.to_string_lossy().to_string();

    quote! {
        const _: &str = include_str!(#path_str);

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum CrousRegion {
            #(#variants),*
        }

        impl CrousRegion {
            pub fn url(&self) -> &'static str {
                match self {
                    #(CrousRegion::#variants => #urls),*
                }
            }

            pub fn to_url(&self) -> ::std::string::String {
                ::std::format!("{}se-restaurer/ou-manger/", self.url())
            }

            pub fn all() -> &'static [CrousRegion] {
                &[#(CrousRegion::#variants),*]
            }
        }

        impl ::std::fmt::Display for CrousRegion {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let s = match self {
                    #(CrousRegion::#variants => #keys),*
                };
                f.write_str(s)
            }
        }

        impl ::std::str::FromStr for CrousRegion {
            type Err = String;
            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                match s {
                    #(#keys => ::std::result::Result::Ok(CrousRegion::#variants),)*
                    _ => ::std::result::Result::Err(format!("unknown CROUS region: {}", s)),
                }
            }
        }
    }
    .into()
}

/// Reads a JSON file at compile time and generates:
/// - A `CrousData` struct whose fields are the lowercased keys of the JSON object
/// - A `get_urls()` function that returns a `CrousData` populated with the JSON values
///
/// Usage: `generate_crous_data!("src/data/crous.json");`
/// The path is relative to the crate's `CARGO_MANIFEST_DIR`.
#[proc_macro]
pub fn generate_crous_data(input: TokenStream) -> TokenStream {
    let path_lit = parse_macro_input!(input as LitStr);
    let relative_path = path_lit.value();

    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let full_path = std::path::Path::new(&manifest_dir).join(&relative_path);

    let content = std::fs::read_to_string(&full_path).unwrap_or_else(|e| {
        panic!("generate_crous_data: failed to read {}: {}", full_path.display(), e)
    });

    let json: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|e| {
        panic!("generate_crous_data: failed to parse JSON at {}: {}", full_path.display(), e)
    });

    let obj = json.as_object().unwrap_or_else(|| {
        panic!("generate_crous_data: JSON root at {} must be an object", full_path.display())
    });

    let fields: Vec<_> = obj
        .keys()
        .map(|k| {
            let ident = syn::Ident::new(&k.to_lowercase(), Span::call_site());
            quote! { pub #ident: CrousUrl }
        })
        .collect();

    // Each field init reads its value from the parsed JSON at runtime using the original key.
    let field_inits: Vec<_> = obj
        .keys()
        .map(|k| {
            let ident = syn::Ident::new(&k.to_lowercase(), Span::call_site());
            let key = k.as_str();
            quote! {
                #ident: CrousUrl(
                    json[#key]
                        .as_str()
                        .expect(concat!("crous.json: key \"", #key, "\" is missing or not a string"))
                        .to_string()
                )
            }
        })
        .collect();

    // Absolute path embedded via include_str! so rustc tracks the file for incremental recompilation.
    let path_str = full_path.to_string_lossy().to_string();

    quote! {
        pub struct CrousUrl(pub String);

        impl CrousUrl {
            pub fn to_list_url(&self) -> String {
                format!("{}se-restaurer/ou-manger/", self.0)
            }
        }

        impl ::std::ops::Deref for CrousUrl {
            type Target = str;
            fn deref(&self) -> &str {
                &self.0
            }
        }

        pub struct CrousData {
            #(#fields),*
        }

        pub fn get_urls() -> CrousData {
            let content = include_str!(#path_str);
            let json: ::serde_json::Value =
                ::serde_json::from_str(content).expect("crous.json: invalid JSON");
            CrousData {
                #(#field_inits),*
            }
        }
    }
    .into()
}
