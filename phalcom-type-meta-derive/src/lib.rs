//! Schema-derived current-version metadata construction with explicit allocation budgets.
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

fn named(fields: &syn::FieldsNamed, constructor: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let names = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect::<Vec<_>>();
    let vars = names.iter().map(|n| format_ident!("field_{}", n)).collect::<Vec<_>>();
    let types = fields.named.iter().map(|f| &f.ty).collect::<Vec<_>>();
    let defaults = fields
        .named
        .iter()
        .map(|f| {
            let option = matches!(&f.ty,Type::Path(p) if p.path.segments.last().is_some_and(|s|s.ident=="Option"));
            option || f.attrs.iter().any(|a| a.path().is_ident("serde") && quote!(#a).to_string().contains("default"))
        })
        .collect::<Vec<_>>();
    let values = vars
        .iter()
        .zip(names.iter())
        .zip(defaults)
        .map(|((v, n), d)| {
            if d {
                quote!(#v.unwrap_or_default())
            } else {
                let msg = format!("missing metadata field {}", n);
                quote!(#v.ok_or(crate::budgeted::DecodeError::Syntax(#msg))?)
            }
        })
        .collect::<Vec<_>>();
    let strings = names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    quote! {
        #(let mut #vars: Option<#types> = None;)*
        json.cursor.expect(b'{')?;
        let mut first=true;
        while let Some(key)=json.cursor.object_key(&mut first)? {
            #(if key.equals(#strings) {if #vars.is_some(){return Err(crate::budgeted::DecodeError::Syntax("duplicate metadata field"));}#vars=Some(<#types as crate::budgeted::BudgetDecode>::decode(json)?);continue;})*
            json.cursor.skip_value()?;
        }
        Ok(#constructor { #(#names: #values),* })
    }
}
fn fields(fields: &Fields, constructor: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(n) => named(n, constructor),
        Fields::Unnamed(u) => {
            let tys = u.unnamed.iter().map(|f| &f.ty).collect::<Vec<_>>();
            if tys.len() == 1 {
                let ty = tys[0];
                quote!(Ok(#constructor(<#ty as crate::budgeted::BudgetDecode>::decode(json)?)))
            } else {
                let vars = (0..tys.len()).map(|i| format_ident!("element_{i}")).collect::<Vec<_>>();
                let commas = (0..tys.len())
                    .map(|i| if i == 0 { quote!() } else { quote!(json.cursor.expect(b',')?;) })
                    .collect::<Vec<_>>();
                quote!(json.cursor.expect(b'[')?;#(#commas let #vars=<#tys as crate::budgeted::BudgetDecode>::decode(json)?;)*json.cursor.expect(b']')?;Ok(#constructor(#(#vars),*)))
            }
        }
        Fields::Unit => quote!(Ok(#constructor)),
    }
}
#[proc_macro_derive(BudgetDecode)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let body = match &input.data {
        Data::Struct(s) => fields(&s.fields, quote!(Self)),
        Data::Enum(e) => {
            let units = e
                .variants
                .iter()
                .filter(|v| matches!(v.fields, Fields::Unit))
                .map(|v| {
                    let n = &v.ident;
                    let tag = n.to_string();
                    quote!(if tag.equals(#tag){return Ok(Self::#n);})
                })
                .collect::<Vec<_>>();
            let nonunits=e.variants.iter().filter(|v|!matches!(v.fields,Fields::Unit)).map(|v|{let n=&v.ident;let tag=n.to_string();let inner=fields(&v.fields,quote!(Self::#n));quote!(if tag.equals(#tag){let value=(|| -> Result<Self,crate::budgeted::DecodeError> {#inner})()?;json.cursor.expect(b'}')?;return Ok(value);})}).collect::<Vec<_>>();
            quote! {
                if json.cursor.peek()==Some(b'"'){let tag=json.cursor.string_token()?;#(#units)*return Err(crate::budgeted::DecodeError::Syntax("unknown metadata enum tag"));}
                json.cursor.expect(b'{')?;let tag=json.cursor.string_token()?;json.cursor.expect(b':')?;
                #(#nonunits)*
                Err(crate::budgeted::DecodeError::Syntax("unknown metadata enum tag"))
            }
        }
        Data::Union(_) => return syn::Error::new_spanned(&input, "metadata unions are unsupported").to_compile_error().into(),
    };
    quote!(impl crate::budgeted::BudgetDecode for #name {fn decode(json:&mut crate::budgeted::Decoder<'_, '_>)->Result<Self,crate::budgeted::DecodeError>{json.nested(|json|{#body})}}).into()
}
