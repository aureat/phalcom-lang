//! Procedural macro `#[primitive]` for native Phalcom primitives.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Error, Expr, Ident, ItemFn, LitStr, Result, Token, parse_macro_input};

use phalcom_common::selector::{Selector, SelectorKind, SelectorSlot};
use phalcom_native_meta::UniverseKey;
use phalcom_type_syntax::{CallableType, ParameterTuple, TypeExpr, parse_callable_type, parse_type_expr};

struct PrimitiveAttrArgs {
    _owner: UniverseKey,
    owner_ident: Ident,
    selector_str: String,
    selector_span: Span,

    params_str: Option<(String, Span)>,
    returns_str: Option<(String, Span)>,
    types_str: Option<(String, Span)>,

    raises_expr: Option<Expr>,
    effects_expr: Option<Expr>,

    side: Option<String>,
    visibility: Option<String>,
    stability: Option<String>,

    since: Option<String>,
    deprecated_since: Option<String>,
    replacement: Option<String>,

    abi: Option<String>,
    flow: Option<String>,

    intrinsic: Option<String>,
    trust: Option<String>,
}

impl Parse for PrimitiveAttrArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let owner_ident: Ident = input.parse()?;
        let owner = UniverseKey::from_name(&owner_ident.to_string())
            .ok_or_else(|| Error::new(owner_ident.span(), format!("unknown universe owner '{}'", owner_ident)))?;

        input.parse::<Token![,]>()?;
        let selector_lit: LitStr = input.parse()?;
        let selector_str = selector_lit.value();
        let selector_span = selector_lit.span();

        let mut args = PrimitiveAttrArgs {
            _owner: owner,
            owner_ident,
            selector_str,
            selector_span,
            params_str: None,
            returns_str: None,
            types_str: None,
            raises_expr: None,
            effects_expr: None,
            side: None,
            visibility: None,
            stability: None,
            since: None,
            deprecated_since: None,
            replacement: None,
            abi: None,
            flow: None,
            intrinsic: None,
            trust: None,
        };

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            let field_ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let field_name = field_ident.to_string();

            match field_name.as_str() {
                "params" => {
                    let content;
                    syn::bracketed!(content in input);
                    let mut elements = Vec::new();
                    while !content.is_empty() {
                        // Check if it's `label: Type` or `Type`
                        if content.peek2(Token![:]) {
                            let label: Ident = content.parse()?;
                            content.parse::<Token![:]>()?;
                            let ty_str: TokenStream2 = content.parse::<syn::Type>()?.into_token_stream();
                            elements.push(format!("{}: {}", label, ty_str));
                        } else if content.peek(Token![...]) {
                            content.parse::<Token![...]>()?;
                            if !content.is_empty() && !content.peek(Token![,]) {
                                let ty_str: TokenStream2 = content.parse::<syn::Type>()?.into_token_stream();
                                elements.push(format!("...{}", ty_str));
                            } else {
                                elements.push("...".to_string());
                            }
                        } else {
                            let ty_str: TokenStream2 = content.parse::<syn::Type>()?.into_token_stream();
                            elements.push(ty_str.to_string());
                        }
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        }
                    }
                    args.params_str = Some((format!("({})", elements.join(", ")), field_ident.span()));
                }
                "returns" => {
                    if input.peek(LitStr) {
                        let lit: LitStr = input.parse()?;
                        args.returns_str = Some((lit.value(), lit.span()));
                    } else {
                        let ty: syn::Type = input.parse()?;
                        args.returns_str = Some((quote!(#ty).to_string(), field_ident.span()));
                    }
                }
                "types" => {
                    let lit: LitStr = input.parse()?;
                    args.types_str = Some((lit.value(), lit.span()));
                }
                "raises" => {
                    let expr: Expr = input.parse()?;
                    args.raises_expr = Some(expr);
                }
                "effects" => {
                    let expr: Expr = input.parse()?;
                    args.effects_expr = Some(expr);
                }
                "side" => {
                    let id: Ident = input.parse()?;
                    args.side = Some(id.to_string());
                }
                "visibility" => {
                    let id: Ident = input.parse()?;
                    args.visibility = Some(id.to_string());
                }
                "stability" => {
                    let id: Ident = input.parse()?;
                    args.stability = Some(id.to_string());
                }
                "since" => {
                    let lit: LitStr = input.parse()?;
                    args.since = Some(lit.value());
                }
                "deprecated_since" => {
                    let lit: LitStr = input.parse()?;
                    args.deprecated_since = Some(lit.value());
                }
                "replacement" => {
                    let lit: LitStr = input.parse()?;
                    args.replacement = Some(lit.value());
                }
                "abi" => {
                    let id: Ident = input.parse()?;
                    args.abi = Some(id.to_string());
                }
                "flow" => {
                    let flow_expr: Expr = input.parse()?;
                    args.flow = Some(quote!(#flow_expr).to_string());
                }
                "intrinsic" => {
                    let id: Ident = input.parse()?;
                    args.intrinsic = Some(id.to_string());
                }
                "trust" => {
                    let id: Ident = input.parse()?;
                    args.trust = Some(id.to_string());
                }
                other => {
                    return Err(Error::new(field_ident.span(), format!("unknown primitive attribute field '{other}'")));
                }
            }
        }

        Ok(args)
    }
}

#[proc_macro_attribute]
pub fn primitive(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as PrimitiveAttrArgs);
    let item_fn = parse_macro_input!(input as ItemFn);

    match expand_primitive(attr_args, &item_fn) {
        Ok(ts) => ts.into(),
        Err(err) => {
            let compile_err = err.to_compile_error();
            let expanded = quote! {
                #item_fn
                #compile_err
            };
            expanded.into()
        }
    }
}

fn expand_primitive(args: PrimitiveAttrArgs, item_fn: &ItemFn) -> Result<TokenStream2> {
    let fn_name = &item_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // 1. Validate selector
    let selector =
        Selector::try_decode_exact(&args.selector_str).map_err(|e| Error::new(args.selector_span, format!("invalid selector '{}': {e}", args.selector_str)))?;
    let canonical_str = selector.encode();
    if canonical_str != args.selector_str {
        return Err(Error::new(
            args.selector_span,
            format!("noncanonical selector spelling '{}', expected canonical '{}'", args.selector_str, canonical_str),
        ));
    }

    // 2. Validate visibility and _$ prefix
    let is_internal_sel = args.selector_str.starts_with("_$");
    let visibility = match args.visibility.as_deref() {
        Some("public") => phalcom_native_meta::NativeVisibility::Public,
        Some("internal") => phalcom_native_meta::NativeVisibility::Internal,
        Some(other) => {
            return Err(Error::new(
                Span::call_site(),
                format!("unknown visibility '{other}', expected public or internal"),
            ));
        }
        None => {
            if is_internal_sel {
                return Err(Error::new(
                    args.selector_span,
                    "selector beginning with '_$' requires explicit visibility (visibility = internal | public)",
                ));
            }
            phalcom_native_meta::NativeVisibility::Public
        }
    };
    if visibility == phalcom_native_meta::NativeVisibility::Internal && !is_internal_sel {
        return Err(Error::new(args.selector_span, "visibility = internal requires selector to begin with '_$'"));
    }

    // 3. Dispatch side
    let side = match args.side.as_deref() {
        Some("instance") | None => phalcom_native_meta::NativeDispatch::Instance,
        Some("class") => phalcom_native_meta::NativeDispatch::Class,
        Some(other) => {
            return Err(Error::new(Span::call_site(), format!("unknown side '{other}', expected instance or class")));
        }
    };

    // 4. Stability
    let stability = match args.stability.as_deref() {
        Some("unspecified") | None => phalcom_native_meta::NativeStability::Unspecified,
        Some("experimental") => phalcom_native_meta::NativeStability::Experimental,
        Some("stable") => phalcom_native_meta::NativeStability::Stable,
        Some(other) => {
            return Err(Error::new(Span::call_site(), format!("unknown stability '{other}'")));
        }
    };

    // 5. ABI
    let abi = match args.abi.as_deref() {
        Some("value") | None => phalcom_native_meta::PrimitiveAbi::Value,
        Some("shape") => phalcom_native_meta::PrimitiveAbi::Shape,
        Some(other) => {
            return Err(Error::new(Span::call_site(), format!("unknown abi '{other}', expected value or shape")));
        }
    };

    // 6. Trust
    let trust = match args.trust.as_deref() {
        Some("ordinary") | None => phalcom_native_meta::NativeTrust::Ordinary,
        Some("privileged") => phalcom_native_meta::NativeTrust::Privileged,
        Some(other) => {
            return Err(Error::new(
                Span::call_site(),
                format!("unknown trust '{other}', expected ordinary or privileged"),
            ));
        }
    };

    // 7. Intrinsic
    let intrinsic_tokens = match args.intrinsic.as_deref() {
        Some("BoolAnd") => quote!(Some(::phalcom_native_meta::NativeIntrinsicId::BoolAnd)),
        Some("BoolOr") => quote!(Some(::phalcom_native_meta::NativeIntrinsicId::BoolOr)),
        Some("BoolNot") => quote!(Some(::phalcom_native_meta::NativeIntrinsicId::BoolNot)),
        Some(other) => {
            return Err(Error::new(Span::call_site(), format!("unknown intrinsic ID '{other}'")));
        }
        None => quote!(None),
    };

    // 8. Lifecycle
    let since_tokens = match &args.since {
        Some(s) => quote!(Some(#s)),
        None => quote!(None),
    };
    let deprecated_since_tokens = match &args.deprecated_since {
        Some(s) => quote!(Some(#s)),
        None => quote!(None),
    };
    let replacement_tokens = match &args.replacement {
        Some(s) => {
            if args.deprecated_since.is_none() {
                return Err(Error::new(Span::call_site(), "replacement selector requires deprecated_since metadata"));
            }
            quote!(Some(#s))
        }
        None => quote!(None),
    };

    // 9. Flow
    let flow_tokens = match args.flow.as_deref() {
        Some("value") | None => quote!(::phalcom_native_meta::ReturnFlowSpec::Value),
        Some("receiver") => quote!(::phalcom_native_meta::ReturnFlowSpec::Receiver),
        Some("never") => quote!(::phalcom_native_meta::ReturnFlowSpec::Never),
        Some("unknown") => quote!(::phalcom_native_meta::ReturnFlowSpec::Unknown),
        Some(s) if s.starts_with("argument(") && s.ends_with(')') => {
            let inner = &s["argument(".len()..s.len() - 1].trim();
            let idx: usize = inner
                .parse()
                .map_err(|_| Error::new(Span::call_site(), format!("invalid flow argument index: '{inner}'")))?;
            quote!(::phalcom_native_meta::ReturnFlowSpec::Argument(#idx))
        }
        Some(other) => {
            return Err(Error::new(Span::call_site(), format!("unknown flow spec '{other}'")));
        }
    };

    // 10. Effects
    let effects_tokens = match &args.effects_expr {
        Some(Expr::Path(p)) if p.path.is_ident("pure") => {
            quote!(::phalcom_native_meta::EffectSpec::Pure)
        }
        Some(Expr::Path(p)) if p.path.is_ident("unknown") => {
            quote!(::phalcom_native_meta::EffectSpec::Unknown)
        }
        Some(Expr::Array(arr)) => {
            let mut effect_idents = Vec::new();
            for elem in &arr.elems {
                if let Expr::Path(p) = elem {
                    let id = p.path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                    match id.as_str() {
                        "mutation" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Mutation)),
                        "io" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Io)),
                        "scheduling" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Scheduling)),
                        "reflection" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Reflection)),
                        "nondeterminism" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Nondeterminism)),
                        "blocking" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Blocking)),
                        other => {
                            return Err(Error::new(elem.span(), format!("unknown effect '{other}'")));
                        }
                    }
                } else {
                    return Err(Error::new(elem.span(), "expected effect identifier"));
                }
            }
            quote!(::phalcom_native_meta::EffectSpec::Known(&[#(#effect_idents),*]))
        }
        Some(other) => {
            return Err(Error::new(
                other.span(),
                "invalid effects specification (use 'pure', 'unknown', or [effect, ...])",
            ));
        }
        None => quote!(::phalcom_native_meta::EffectSpec::Unknown),
    };

    // 11. Raises
    let raises_tokens = match &args.raises_expr {
        Some(Expr::Array(arr)) => {
            let mut types = Vec::new();
            for elem in &arr.elems {
                let ty_str = quote!(#elem).to_string();
                let ty_expr = parse_type_expr(&ty_str).map_err(|e| Error::new(elem.span(), format!("failed to parse raise type '{ty_str}': {e}")))?;
                let ty_tokens = emit_type_expr_spec(&ty_expr, &[])?;
                types.push(quote!(*#ty_tokens));
            }
            quote!(::phalcom_native_meta::RaisesSpec::Known(&[#(#types),*]))
        }
        Some(other) => {
            return Err(Error::new(other.span(), "invalid raises specification (use [Type, ...])"));
        }
        None => quote!(::phalcom_native_meta::RaisesSpec::Unknown),
    };

    // 12. Parse callable / params / returns and cross-check
    let (callable_spec, params_spec, returns_spec) = if let Some((types_str, types_span)) = &args.types_str {
        let parsed_callable =
            parse_callable_type(types_str).map_err(|e| Error::new(*types_span, format!("failed to parse types contract '{types_str}': {e}")))?;

        let binders: Vec<String> = parsed_callable.type_params.iter().map(|p| p.name.clone()).collect();

        // If params_str is present, cross-check
        if let Some((params_str, params_span)) = &args.params_str {
            let parsed_params_ty = parse_type_expr(params_str).map_err(|e| Error::new(*params_span, format!("failed to parse params '{params_str}': {e}")))?;
            let TypeExpr::Tuple(parsed_params) = parsed_params_ty else {
                return Err(Error::new(*params_span, "expected tuple-shaped parameter list"));
            };

            // Cross check structural equivalence
            if *parsed_params != parsed_callable.params {
                return Err(Error::new(
                    *params_span,
                    format!(
                        "params mismatch: declared params '{}' does not match types '{}'",
                        parsed_params, parsed_callable.params
                    ),
                ));
            }
        }

        // If returns_str is present, cross-check
        if let Some((returns_str, returns_span)) = &args.returns_str {
            let parsed_returns =
                parse_type_expr(returns_str).map_err(|e| Error::new(*returns_span, format!("failed to parse returns '{returns_str}': {e}")))?;
            if parsed_returns != parsed_callable.return_type {
                return Err(Error::new(
                    *returns_span,
                    format!(
                        "returns mismatch: declared returns '{}' does not match types return '{}'",
                        parsed_returns, parsed_callable.return_type
                    ),
                ));
            }
        }

        // Cross-check selector and params
        cross_check_selector_and_params(&selector, &parsed_callable.params, args.selector_span)?;

        let callable_tokens = emit_callable_spec(&parsed_callable)?;
        let params_tokens = emit_params_spec(&parsed_callable.params, &binders)?;
        let returns_tokens = emit_type_expr_spec(&parsed_callable.return_type, &binders)?;

        (callable_tokens, params_tokens, returns_tokens)
    } else {
        // Compatibility mode when types is omitted
        let default_params = ParameterTuple::default();
        let default_returns = TypeExpr::Unknown;
        let default_callable = CallableType {
            type_params: Vec::new(),
            params: default_params.clone(),
            return_type: default_returns.clone(),
        };
        (
            emit_callable_spec(&default_callable)?,
            emit_params_spec(&default_params, &[])?,
            emit_type_expr_spec(&default_returns, &[])?,
        )
    };

    // Surface spec tokens
    let owner_key_ident = &args.owner_ident;
    let selector_str = &args.selector_str;
    let visibility_tokens = match visibility {
        phalcom_native_meta::NativeVisibility::Public => quote!(::phalcom_native_meta::NativeVisibility::Public),
        phalcom_native_meta::NativeVisibility::Internal => quote!(::phalcom_native_meta::NativeVisibility::Internal),
    };
    let side_tokens = match side {
        phalcom_native_meta::NativeDispatch::Instance => quote!(::phalcom_native_meta::NativeDispatch::Instance),
        phalcom_native_meta::NativeDispatch::Class => quote!(::phalcom_native_meta::NativeDispatch::Class),
    };
    let stability_tokens = match stability {
        phalcom_native_meta::NativeStability::Unspecified => quote!(::phalcom_native_meta::NativeStability::Unspecified),
        phalcom_native_meta::NativeStability::Experimental => quote!(::phalcom_native_meta::NativeStability::Experimental),
        phalcom_native_meta::NativeStability::Stable => quote!(::phalcom_native_meta::NativeStability::Stable),
    };
    let trust_tokens = match trust {
        phalcom_native_meta::NativeTrust::Ordinary => quote!(::phalcom_native_meta::NativeTrust::Ordinary),
        phalcom_native_meta::NativeTrust::Privileged => quote!(::phalcom_native_meta::NativeTrust::Privileged),
    };

    let abi_tokens = match abi {
        phalcom_native_meta::PrimitiveAbi::Value => quote!(::phalcom_native_meta::PrimitiveAbi::Value),
        phalcom_native_meta::PrimitiveAbi::Shape => quote!(::phalcom_native_meta::PrimitiveAbi::Shape),
    };

    let abi_check = match abi {
        phalcom_native_meta::PrimitiveAbi::Value => {
            quote! {
                const _: crate::native::PrimitiveValueFn = #fn_name;
            }
        }
        phalcom_native_meta::PrimitiveAbi::Shape => {
            quote! {
                const _: crate::native::PrimitiveShapeFn = #fn_name;
            }
        }
    };

    let entry_tokens = match abi {
        phalcom_native_meta::PrimitiveAbi::Value => {
            quote!(crate::native::PrimitiveEntry::Value(#fn_name))
        }
        phalcom_native_meta::PrimitiveAbi::Shape => {
            quote!(crate::native::PrimitiveEntry::Shape(#fn_name))
        }
    };

    let fn_name_upper = fn_name_str.to_ascii_uppercase();
    let descriptor_ident = format_ident!("__PHALCOM_PRIMITIVE_DESCRIPTOR_{}", fn_name_upper);
    let surface_ident = format_ident!("__PHALCOM_PRIMITIVE_SURFACE_{}", fn_name_upper);

    let expanded = quote! {
        #item_fn

        #abi_check

        #[allow(non_upper_case_globals)]
        pub static #surface_ident: ::phalcom_native_meta::PrimitiveSurfaceSpec = ::phalcom_native_meta::PrimitiveSurfaceSpec {
            key: ::phalcom_native_meta::PrimitiveKey {
                owner: ::phalcom_native_meta::UniverseKey::#owner_key_ident,
                side: #side_tokens,
                selector: #selector_str,
            },
            visibility: #visibility_tokens,
            stability: #stability_tokens,
            params: #params_spec,
            returns: #returns_spec,
            callable: #callable_spec,
            raises: #raises_tokens,
            effects: #effects_tokens,
            flow: #flow_tokens,
            since: #since_tokens,
            deprecated_since: #deprecated_since_tokens,
            replacement: #replacement_tokens,
            intrinsic: #intrinsic_tokens,
            trust: #trust_tokens,
        };

        #[::linkme::distributed_slice(crate::native::PRIMITIVES)]
        static #descriptor_ident: crate::native::PrimitiveDescriptor = crate::native::PrimitiveDescriptor {
            surface: &#surface_ident,
            abi: #abi_tokens,
            entry: #entry_tokens,
            source: ::phalcom_native_meta::NativeSourceSpec {
                module_path: module_path!(),
                rust_name: #fn_name_str,
                file: file!(),
                line: line!(),
            },
        };
    };

    Ok(expanded)
}

fn cross_check_selector_and_params(selector: &Selector, params: &ParameterTuple, span: Span) -> Result<()> {
    let mut expected_pos = 0;
    let mut expected_labels = Vec::new();

    for slot in selector.slots.iter() {
        match slot {
            SelectorSlot::Positional => expected_pos += 1,
            SelectorSlot::Label(l) => expected_labels.push(l.clone()),
        }
    }

    if matches!(selector.kind, SelectorKind::Setter | SelectorKind::SubscriptSet) {
        expected_pos += 1;
    }

    if params.positional.len() != expected_pos {
        return Err(Error::new(
            span,
            format!(
                "selector positional count mismatch: selector '{}' expects {} positionals, params has {}",
                selector.encode(),
                expected_pos,
                params.positional.len()
            ),
        ));
    }

    if params.labeled.len() != expected_labels.len() {
        return Err(Error::new(
            span,
            format!(
                "selector labeled count mismatch: selector '{}' expects {} labels, params has {}",
                selector.encode(),
                expected_labels.len(),
                params.labeled.len()
            ),
        ));
    }

    for (i, (expected_label, actual_labeled)) in expected_labels.iter().zip(&params.labeled).enumerate() {
        if expected_label != &actual_labeled.label {
            return Err(Error::new(
                span,
                format!(
                    "selector label mismatch at slot {i}: selector has '{expected_label}', params has '{actual_label}'",
                    actual_label = actual_labeled.label
                ),
            ));
        }
    }

    Ok(())
}

fn emit_type_expr_spec(ty: &TypeExpr, binders: &[String]) -> Result<TokenStream2> {
    match ty {
        TypeExpr::Unknown => Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Unknown)),
        TypeExpr::Never => Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Never)),
        TypeExpr::SelfType => Ok(quote!(&::phalcom_native_meta::TypeExprSpec::SelfType)),
        TypeExpr::Universe(name) => {
            let key = UniverseKey::from_name(name).ok_or_else(|| Error::new(Span::call_site(), format!("unknown universe type 'universe.{name}'")))?;
            let key_ident = format_ident!("{}", key.name());
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Universe(::phalcom_native_meta::UniverseKey::#key_ident)))
        }
        TypeExpr::Named(name) => {
            if binders.contains(name) {
                let name_str = name.as_str();
                Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Parameter(#name_str)))
            } else if let Some(key) = UniverseKey::from_name(name) {
                let key_ident = format_ident!("{}", key.name());
                Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Universe(::phalcom_native_meta::UniverseKey::#key_ident)))
            } else {
                Err(Error::new(
                    Span::call_site(),
                    format!("unknown type name '{name}': not a UniverseKey or in-scope generic type parameter"),
                ))
            }
        }
        TypeExpr::Parameter(name) => {
            let name_str = name.as_str();
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Parameter(#name_str)))
        }
        TypeExpr::Applied { origin, arguments } => {
            let origin_tokens = emit_type_expr_spec(origin, binders)?;
            let mut arg_tokens = Vec::new();
            for arg in arguments {
                let elem = emit_type_expr_spec(arg, binders)?;
                arg_tokens.push(quote!(*#elem));
            }
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Applied {
                origin: #origin_tokens,
                arguments: &[#(#arg_tokens),*],
            }))
        }
        TypeExpr::Union(alts) => {
            let mut alt_tokens = Vec::new();
            for alt in alts {
                let elem = emit_type_expr_spec(alt, binders)?;
                alt_tokens.push(quote!(*#elem));
            }
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Union(&[#(#alt_tokens),*])))
        }
        TypeExpr::Tuple(tuple) => {
            let tuple_spec = emit_params_spec(tuple, binders)?;
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Tuple(#tuple_spec)))
        }
    }
}

fn emit_params_spec(params: &ParameterTuple, binders: &[String]) -> Result<TokenStream2> {
    let mut pos_tokens = Vec::new();
    for pos in &params.positional {
        let elem = emit_type_expr_spec(pos, binders)?;
        pos_tokens.push(quote!(*#elem));
    }

    let mut labeled_tokens = Vec::new();
    for labeled in &params.labeled {
        let label_str = &labeled.label;
        let elem = emit_type_expr_spec(&labeled.ty, binders)?;
        labeled_tokens.push(quote!(::phalcom_native_meta::LabeledParameterSpec {
            label: #label_str,
            ty: #elem,
        }));
    }

    let rest_tokens = match &params.rest {
        Some(rest) => {
            let ty_tokens = match &rest.ty {
                Some(ty) => {
                    let elem = emit_type_expr_spec(ty, binders)?;
                    quote!(Some(#elem))
                }
                None => quote!(None),
            };
            quote!(Some(::phalcom_native_meta::RestParameterSpec { ty: #ty_tokens }))
        }
        None => quote!(None),
    };

    Ok(quote!(&::phalcom_native_meta::ParameterTupleSpec {
        positional: &[#(#pos_tokens),*],
        labeled: &[#(#labeled_tokens),*],
        rest: #rest_tokens,
    }))
}

fn emit_callable_spec(callable: &CallableType) -> Result<TokenStream2> {
    let binders: Vec<String> = callable.type_params.iter().map(|p| p.name.clone()).collect();
    let mut type_param_tokens = Vec::new();
    for p in &callable.type_params {
        let name_str = &p.name;
        type_param_tokens.push(quote!(::phalcom_native_meta::TypeParameterSpec {
            name: #name_str,
            kind: ::phalcom_native_meta::KindSpec::Type,
        }));
    }

    let params_spec = emit_params_spec(&callable.params, &binders)?;
    let returns_spec = emit_type_expr_spec(&callable.return_type, &binders)?;

    Ok(quote!(&::phalcom_native_meta::CallableTypeSpec {
        type_params: &[#(#type_param_tokens),*],
        params: #params_spec,
        return_type: #returns_spec,
    }))
}
