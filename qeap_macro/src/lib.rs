// going through youtube tutorial first because I like the way this guy handles proc macros...

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    Attribute, DeriveInput, Expr, GenericArgument, Ident, ItemFn, PatType, PathArguments,
    PathSegment, ReturnType, Token, Type, TypeReference, parse::Parse, parse_macro_input,
};

use quote::{ToTokens, quote};

struct QeapAttributes {
    with: Option<Expr>,
}

impl QeapAttributes {
    fn parse(attrs: &[Attribute]) -> Self {
        let mut qeap_attrs = Self { with: None };

        for attr in attrs {
            if !attr.path().is_ident("qeap") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("with") {
                    let _ = meta.input.parse::<Token![=]>()?;

                    qeap_attrs.with = Some(meta.input.parse::<Expr>()?);
                }

                Ok(())
            })
            .expect("with = <expr>");
        }

        qeap_attrs
    }
}

#[proc_macro_derive(Qeap, attributes(qeap))]
pub fn derive_qeap(input: TokenStream) -> TokenStream {
    let c = parse_macro_input!(input as DeriveInput);

    let qeap_attrs = QeapAttributes::parse(&c.attrs);

    let type_name = &c.ident;
    let type_name_str = c.ident.to_string();

    let persistence_mechanism_create = qeap_attrs.with.expect("with = <expr> is required");

    let out = quote! {
        impl qeap::Qeap for #type_name {
            fn load() -> qeap::QeapResult<Self>
            where
                Self: Sized
            {
                let p = #persistence_mechanism_create;
                ::qeap::PersistenceMechanism::init(&p)?;
                ::qeap::PersistenceMechanism::load(&p, #type_name_str)
            }

            fn save(&self) -> qeap::QeapResult<()> {
                let p = #persistence_mechanism_create;
                ::qeap::PersistenceMechanism::init(&p)?;
                ::qeap::PersistenceMechanism::save(&p, self, #type_name_str)
            }
        }
    };

    out.into()
}

#[proc_macro_attribute]
pub fn scoped(attr: TokenStream, item: TokenStream) -> TokenStream {
    let scoped_mode = parse_macro_input!(attr as ScopedMode);
    let func = parse_macro_input!(item as syn::ItemFn);
    let scoped_fn = create_scoped_fn(scoped_mode, func);

    let out = quote! { #scoped_fn };

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
            VarType::Handle(_) => quote! { ::qeap::Handle::new_handle(&#name) },
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

enum VarType {
    ImmutableRef(TypeReference),
    MutableRef(TypeReference),
    Handle(Type),
}

#[derive(Default, Clone, Copy)]
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
                    format!(
                        "Expected 'flatten', 'flatten_erased', 'absorb', or 'expect', got '{other}'"
                    ),
                ));
            }
        };

        Ok(mode)
    }
}

fn get_result_path_segment(ty: &Type) -> Option<&PathSegment> {
    if let Type::Path(type_path) = ty {
        let seg = type_path.path.segments.last()?;

        if seg.ident.to_string().contains("Result") {
            Some(seg)
        } else {
            None
        }
    } else {
        None
    }
}

fn extract_result_ok_err_types(result_seg: &PathSegment) -> (&Type, &Type) {
    if let PathArguments::AngleBracketed(args) = &result_seg.arguments {
        let mut iter = args.args.iter();

        let ok_ty = match iter.next() {
            Some(GenericArgument::Type(ok_ty)) => ok_ty,
            _ => panic!(
                "If a Result type is specified, both Ok and Err types (T and E) must be included in the signature"
            ),
        };

        let err_ty = match iter.next() {
            Some(GenericArgument::Type(err_ty)) => err_ty,
            _ => panic!(
                "If a Result type is specified, both Ok and Err types (T and E) must be included in the signature"
            ),
        };

        return (ok_ty, err_ty);
    } else {
        panic!(
            "If a Result type is specified, both Ok and Err types (T and E) must be included in the signature"
        );
    }
}

fn gather_scoped_fields(func: &ItemFn) -> Vec<ScopeField> {
    func.sig
        .inputs
        .iter()
        .filter_map(|a| match a {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pat_type) => Some(pat_type),
        })
        .map(ScopeField::from)
        .collect()
}

struct ScopedFn {
    scoped_mode: ScopedMode,
    scoped_fields: Vec<ScopeField>,
    output: proc_macro2::TokenStream,
    og_func: ItemFn,
}

impl ToTokens for ScopedFn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let var_use = self.scoped_fields.iter().map(ScopeField::as_var_use);
        let field_decls = self
            .scoped_fields
            .iter()
            .map(ScopeField::as_field_declaration);
        let field_names = self.scoped_fields.iter().map(|f| &f.name);

        let func_name = &self.og_func.sig.ident;

        let inner_func_name = Ident::new(&format!("{}_inner", func_name), Span::call_site());

        let mut func = self.og_func.clone();

        let return_expr = &self.output;

        func.sig.ident = inner_func_name.clone();

        let t = match self.scoped_mode {
            ScopedMode::Nested => {
                quote! {
                    fn #func_name() -> #return_expr {
                        #func
                        #(
                            #field_decls = ::qeap::Qeap::load()?;
                            )*

                            let result = #inner_func_name(#(#var_use),*);

                        #(
                            ::qeap::Qeap::save(&#field_names)?;
                        )*

                        Ok(result)
                    }
                }
            }
            ScopedMode::Absorb => {
                quote! {
                    fn #func_name() -> #return_expr {
                        #func
                        #(
                            #field_decls = ::qeap::Qeap::load()?;
                            )*

                            let result = #inner_func_name(#(#var_use),*);

                        #(
                            ::qeap::Qeap::save(&#field_names)?;
                        )*

                        result
                    }
                }
            }
            ScopedMode::Flatten => {
                quote! {
                    fn #func_name() -> #return_expr {
                        #func
                        #(
                            #field_decls = ::qeap::Qeap::load()?;
                        )*

                        let result = ::qeap::transform::IntoFlattenedResult::into_flattened(#inner_func_name(#(#var_use),*));

                        #(
                            ::qeap::Qeap::save(&#field_names)?;
                        )*

                        result
                    }
                }
            }
            ScopedMode::Expect => {
                let expect_save_msg = self
                    .scoped_fields
                    .iter()
                    .map(|f| format!("{} data should save successfully", f.name));
                let expect_load_msg = self
                    .scoped_fields
                    .iter()
                    .map(|f| format!("{} data should load successfully", f.name));

                quote! {
                    fn #func_name() -> #return_expr {
                        #func
                        #(
                            #field_decls = ::qeap::Qeap::load().expect(#expect_load_msg);
                            )*

                            let result = #inner_func_name(#(#var_use),*);

                        #(
                            ::qeap::Qeap::save(&#field_names).expect(#expect_save_msg);
                        )*

                        result
                    }
                }
            }
        };

        tokens.extend(t);
    }
}

fn create_scoped_fn(scoped_mode: ScopedMode, func: ItemFn) -> ScopedFn {
    let scoped_fields = gather_scoped_fields(&func);

    let output = determine_scoped_fn_output(scoped_mode, &func.sig.output);

    ScopedFn {
        scoped_mode,
        output,
        scoped_fields,
        og_func: func,
    }
}

fn determine_scoped_fn_output(
    scoped_mode: ScopedMode,
    original_return_type: &ReturnType,
) -> proc_macro2::TokenStream {
    let original_output = match original_return_type {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    match scoped_mode {
        ScopedMode::Nested => {
            quote! { ::std::result::Result<#original_output, ::qeap::error::Error> }
        }
        ScopedMode::Flatten => match original_return_type {
            ReturnType::Default => quote! {
                ::std::result::Result<(), ::qeap::error::FlattenedError<::std::convert::Infallible>>
            },
            ReturnType::Type(_, ty) => {
                let (ok_ty, err_ty) = match get_result_path_segment(ty) {
                    Some(seg) => {
                        let (ok_ty, err_ty) = extract_result_ok_err_types(seg);
                        (quote! {#ok_ty}, quote! {#err_ty})
                    }
                    None => (quote! { #ty }, quote! { ::std::convert::Infallible }),
                };
                quote! {
                    ::std::result::Result<#ok_ty, ::qeap::error::FlattenedError<#err_ty>>
                }
            }
        },
        ScopedMode::Absorb | ScopedMode::Expect => quote! { #original_output },
    }
}
