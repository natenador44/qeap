// going through youtube tutorial first because I like the way this guy handles proc macros...

use proc_macro::TokenStream;
use syn::{
    DeriveInput, Expr, Ident, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};

struct QeapAttributes {
    dir: Option<Expr>,
}

#[proc_macro_derive(Qeap, attributes(qeap))]
pub fn derive_qeap(input: TokenStream) -> TokenStream {
    let c = parse_macro_input!(input as DeriveInput);

    let mut qeap_attrs = QeapAttributes { dir: None };

    // parse attributes to see if a file name is specified
    for attr in &c.attrs {
        println!("{}", attr.to_token_stream());
        if !attr.path().is_ident("qeap") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            println!("parsing nested meta: {:?}", meta.path.get_ident());
            if meta.path.is_ident("dir") {
                let _ = meta.input.parse::<Token![=]>()?;

                qeap_attrs.dir = Some(meta.input.parse::<Expr>()?);
            }

            Ok(())
        })
        .expect("Expected dir = <expr>");
    }

    let type_name = &c.ident;
    let file_name = format!("{}.json", type_name);

    let root_dir_name = Ident::new(&format!("{type_name}_ROOT_DIR"), Span::call_site());

    let root_dir = qeap_attrs
        .dir
        .expect("`dir` is required: #[qeap(dir = <expr>)]");

    let out = quote! {
        static #root_dir_name: std::sync::LazyLock<std::path::PathBuf> = std::sync::LazyLock::new(|| std::path::PathBuf::from(#root_dir));
        impl #type_name {
            fn init() -> Result<(), qeap::error::InitError> {
                std::fs::create_dir_all(&*#root_dir_name)?;
                Ok(())
            }

            pub fn file_path() -> std::path::PathBuf {
                #root_dir_name.join(Self::FILE_NAME)
            }
        }
        impl qeap::Qeap for #type_name {
            const FILE_NAME: &str = #file_name;

            fn load() -> qeap::QeapLoadResult<Self>
            where
                Self: Sized
            {
                let path = Self::file_path();

                Self::init()?;

                if !path.exists() {
                    let value = Self::default();
                    qeap::save::json(path, &value)?;
                    Ok(value)
                } else {
                    qeap::load::json(path)
                }
            }

            fn save(&self) -> qeap::QeapSaveResult<()> {
                Self::init()?;
                let path = Self::file_path();

                qeap::save::json(path, self)
            }
        }
    };

    out.into()
}
