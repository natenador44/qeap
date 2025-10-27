// going through youtube tutorial first because I like the way this guy handles proc macros...

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    Attribute, DeriveInput, Expr, Ident, PatType, Token, Type, TypeReference, parse::Parse,
    parse_macro_input,
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

            fn load() -> qeap::QeapResult<Self>
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
    ref_type: &'a VarType,
}

impl ToTokens for VarUse<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        let as_tokens = match self.ref_type {
            VarType::ImmutableRef(_) => quote! { &#name },
            VarType::MutableRef(_) => quote! { &mut #name },
            VarType::Handle(_) => quote! { qeap::Handle::new_handle(&#name) },
        };

        tokens.extend(as_tokens);
    }
}

struct FieldDeclaration<'a> {
    name: &'a Ident,
    var_type: &'a VarType,
}

impl ToTokens for FieldDeclaration<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        let ty = match &self.var_type {
            VarType::ImmutableRef(r) | VarType::MutableRef(r) => &*r.elem,
            VarType::Handle(t) => t,
        };

        let as_tokens = match self.var_type {
            VarType::ImmutableRef(_) | VarType::Handle(_) => quote! {
                let #name: #ty
            },
            VarType::MutableRef(_) => {
                quote! { let mut #name: #ty }
            }
        };

        tokens.extend(as_tokens);
    }
}

struct ScopeField {
    name: Ident,
    var_type: VarType,
}

impl ScopeField {
    fn as_var_use(&self) -> VarUse<'_> {
        VarUse {
            name: &self.name,
            ref_type: &self.var_type,
        }
    }

    fn as_field_declaration(&self) -> FieldDeclaration<'_> {
        FieldDeclaration {
            name: &self.name,
            var_type: &self.var_type,
        }
    }

    fn as_field_initialization(&self) -> FieldInitialization<'_> {
        FieldInitialization {
            _var_type: &self.var_type,
        }
    }
}

impl From<&PatType> for ScopeField {
    fn from(value: &PatType) -> Self {
        match &*value.pat {
            syn::Pat::Ident(field_name) => {
                let name = field_name.ident.clone();

                let ref_type = match &*value.ty {
                    // by reference and immutable
                    Type::Reference(
                        tr @ TypeReference {
                            mutability: None, ..
                        },
                    ) => VarType::ImmutableRef(tr.clone()),
                    // by reference and mutable
                    Type::Reference(
                        tr @ TypeReference {
                            mutability: Some(_),
                            ..
                        },
                    ) => VarType::MutableRef(tr.clone()),
                    // by value and mutable
                    other => VarType::Handle(other.clone()),
                };

                Self {
                    name,
                    var_type: ref_type,
                }
            }
            other => panic!(
                "Only ident pattern function arguments are supported, i.e. `field: Type`: {other:?}"
            ),
        }
    }
}

struct FieldInitialization<'a> {
    _var_type: &'a VarType,
}

impl ToTokens for FieldInitialization<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote! { qeap::Qeap::load()? });
    }
}

enum VarType {
    ImmutableRef(TypeReference),
    MutableRef(TypeReference),
    Handle(Type),
}

#[derive(Default)]
enum ScopedMode {
    #[default]
    Nested,
    Flatten,
    Absorb,
    Expect,
}

impl Parse for ScopedMode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let ident: Ident = input.parse()?;

        let mode = match ident.to_string().to_lowercase().as_str() {
            "flatten" => Self::Flatten,
            "absorb" => Self::Absorb,
            "expect" => Self::Expect,
            other => {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("Expected 'flatten', 'absorb', or 'expect', got '{other}'"),
                ));
            }
        };

        Ok(mode)
    }
}

#[proc_macro_attribute]
pub fn scoped(attr: TokenStream, item: TokenStream) -> TokenStream {
    let scoped_mode = parse_macro_input!(attr as ScopedMode);
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
    let field_inits = scoped_fields
        .iter()
        .map(ScopeField::as_field_initialization);

    let scoped_field_names = scoped_fields.iter().map(|f| &f.name).collect::<Vec<_>>();

    let inner_func_name = Ident::new(&format!("{}_inner", func_name), Span::call_site());

    func.sig.ident = inner_func_name.clone();

    let out = quote! {
        fn #func_name() #return_type {
            #func
            #(
                #field_decls = #field_inits;
            )*

            let result = #inner_func_name(#(#var_use),*);

            #(
                qeap::Qeap::save(&#scoped_field_names)?;
            )*

            return result;
        }
    };

    out.into()
}
