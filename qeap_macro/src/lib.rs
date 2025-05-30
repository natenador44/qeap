// going through youtube tutorial first because I like the way this guy handles proc macros...

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    Attribute, DeriveInput, Expr, Ident, PatType, Token, Type, TypeReference, parse_macro_input,
};

use quote::{ToTokens, quote};

struct QeapAttributes {
    dir: Option<Expr>,
}

impl QeapAttributes {
    fn parse(attrs: &[Attribute]) -> Self {
        let mut qeap_attrs = Self { dir: None };

        for attr in attrs {
            if !attr.path().is_ident("qeap") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("dir") {
                    let _ = meta.input.parse::<Token![=]>()?;

                    qeap_attrs.dir = Some(meta.input.parse::<Expr>()?);
                }

                Ok(())
            })
            .expect("Expected dir = <expr>");
        }

        qeap_attrs
    }
}

#[proc_macro_derive(Qeap, attributes(qeap))]
pub fn derive_qeap(input: TokenStream) -> TokenStream {
    let c = parse_macro_input!(input as DeriveInput);

    let qeap_attrs = QeapAttributes::parse(&c.attrs);

    let type_name = &c.ident;
    let file_name = format!("{}.json", type_name);

    let root_dir = qeap_attrs
        .dir
        .expect("`dir` is required: #[qeap(dir = <expr>)]");

    let out = quote! {
        impl #type_name {
            fn init() -> Result<(), qeap::error::InitError> {
                std::fs::create_dir_all(#root_dir)?;
                Ok(())
            }

            pub fn file_path() -> std::path::PathBuf {
                std::path::PathBuf::from(#root_dir).join(Self::FILE_NAME)
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

struct VarUse<'a> {
    name: &'a Ident,
    ref_type: &'a RefType,
}

impl ToTokens for VarUse<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        let as_tokens = match self.ref_type {
            RefType::Immutable => quote! { &#name },
            RefType::Mutable => quote! { &mut #name },
        };

        tokens.extend(as_tokens);
    }
}

struct FieldDeclaration<'a> {
    name: &'a Ident,
    ty: &'a Type,
    ref_type: &'a RefType,
}

impl ToTokens for FieldDeclaration<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        let ty = self.ty;

        let as_tokens = match self.ref_type {
            RefType::Immutable => quote! {
                let #name: #ty
            },
            RefType::Mutable => {
                quote! { let mut #name: #ty }
            }
        };

        tokens.extend(as_tokens);
    }
}

struct ScopeField {
    name: Ident,
    reference: TypeReference,
    ref_type: RefType,
}

impl ScopeField {
    fn as_var_use(&self) -> VarUse {
        VarUse {
            name: &self.name,
            ref_type: &self.ref_type,
        }
    }

    fn as_field_declaration(&self) -> FieldDeclaration {
        FieldDeclaration {
            name: &self.name,
            ty: &*self.reference.elem,
            ref_type: &self.ref_type,
        }
    }
}

impl From<&PatType> for ScopeField {
    fn from(value: &PatType) -> Self {
        match &*value.pat {
            syn::Pat::Ident(field_name) => {
                let name = field_name.ident.clone();
                let reference;

                let ref_type = match &*value.ty {
                    // by reference and immutable
                    Type::Reference(
                        tr @ TypeReference {
                            mutability: None, ..
                        },
                    ) => {
                        reference = tr.clone();
                        RefType::Immutable
                    }
                    // by reference and mutable
                    Type::Reference(
                        tr @ TypeReference {
                            mutability: Some(_),
                            ..
                        },
                    ) => {
                        reference = tr.clone();
                        RefType::Mutable
                    }
                    // by value and mutable
                    _ => panic!(
                        "Only ident pattern function arguments passed by reference are supported, i.e. `field: &Type` or `field: &mut Type`"
                    ),
                };

                Self {
                    name,
                    reference,
                    ref_type,
                }
            }
            _ => panic!("Only ident pattern function arguments are supported, i.e. `field: Type`"),
        }
    }
}

/// For scopes, we always pass the fields by reference. We only need to differientiate between mutable and not.
enum RefType {
    Immutable,
    Mutable,
}

#[proc_macro_attribute]
pub fn scoped(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as syn::ItemFn);

    let func_name = func.sig.ident.clone();
    let return_type = &func.sig.output;

    let scoped_fields = func
        .sig
        .inputs
        .iter()
        .filter_map(|a| match a {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pat_type) => Some(pat_type),
        })
        .map(ScopeField::from)
        .collect::<Vec<_>>();

    let var_use = scoped_fields.iter().map(ScopeField::as_var_use);
    let field_decls = scoped_fields.iter().map(ScopeField::as_field_declaration);

    let scoped_field_names = scoped_fields.iter().map(|f| &f.name).collect::<Vec<_>>();

    let inner_func_name = Ident::new(&format!("{}_inner", func_name), Span::call_site());

    func.sig.ident = inner_func_name.clone();

    let out = quote! {
        fn #func_name() #return_type {
            #func
            #(
                #field_decls = qeap::Qeap::load()?;
            )*

            let result = #inner_func_name(#(#var_use),*);

            #(
                qeap::Qeap::save(&#scoped_field_names)?;
            )*

            return result;
        }
    };

    println!("{out}");

    out.into()
}
