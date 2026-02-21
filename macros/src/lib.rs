use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{LitStr, parse_macro_input};

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
