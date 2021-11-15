mod validate;

use proc_macro::TokenStream;
use quote::{quote};
use crate::validate::ValidateMeta;

#[proc_macro_derive(FieldsValidate, attributes(valid))]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as syn::DeriveInput);
    // eprintln!("{:#?}", st.generics.split_for_impl());
    match do_expand(&st) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

type StructFields = syn::punctuated::Punctuated<syn::Field,syn::Token!(,)>;

fn get_fields_from_derive_input(di: &syn::DeriveInput) -> syn::Result<&StructFields> {
    if let syn::Data::Struct(
        syn::DataStruct {
            fields: syn::Fields::Named(
                syn::FieldsNamed {
                    named,
                    ..
                }
            ),
            ..
        }
    ) = &di.data {
        Ok(named)
    } else {
        Err(syn::Error::new_spanned(di, "Must define on a Struct, not Enum"))
    }
}

fn get_optional_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { ref path, ..}) = ty {
        if let Some(seg) = path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments {
                        args,
                        ..
                    }
                ) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
                        return Some(inner_ty)
                    }
                }
            }
        }
    }
    None
}

fn get_inner_type(ty: &syn::Type) -> String {
    if let syn::Type::Path(syn::TypePath { ref path, ..}) = ty {
        if let Some(seg) = path.segments.last() {
            return seg.ident.to_string()
        }
    }
    "".to_string()
}

fn valid_expand(fields: &StructFields, struct_ident: &syn::Ident) -> syn::Result<(proc_macro2::TokenStream, (Vec<syn::Ident>, Vec<String>))> {
    let mut final_tokenstream = proc_macro2::TokenStream::new();
    let mut add_big_decimal = false;
    let mut static_ref: (Vec<syn::Ident>, Vec<String>) = (Vec::new(), Vec::new());
    for f in fields.iter() {
        for attr in f.attrs.iter() {
            // eprintln!("{:#?}", f);
            let meta = ValidateMeta::from_valid_attr(attr, struct_ident)?;
            let field_type = get_optional_inner_type(&f.ty);
            let type_ = get_inner_type(field_type.unwrap_or(&f.ty));
            let mt = meta
                .validate_token(
                    &f.ident,
                    field_type.is_some(),
                    &type_,
                    (&mut static_ref.0, &mut static_ref.1)
                );
            let msg = meta.err_msg;
            if !add_big_decimal && type_ == "BigDecimal" {
                final_tokenstream.extend(quote!(use ::bigdecimal::ToPrimitive;));
                add_big_decimal = true;
            }
            final_tokenstream.extend(quote! {
                if #mt {
                    return std::result::Result::Err(#msg);
                }
            });
        }
    }
    Ok((final_tokenstream, static_ref))
}

fn do_expand(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = &st.ident;  // 模板代码中不可以使用`.`来访问结构体成员，所以要在模板代码外面将标识符放到一个独立的变量中
    let fields = get_fields_from_derive_input(st)?;
    let (ts, (rex_names, rex)) = valid_expand(fields, struct_ident)?;
    let (impl_generics, type_generics, where_clause) = st.generics.split_for_impl();
    Ok(
        quote! {
            fields_valid::lazy_static! {
                #(static ref #rex_names: fields_valid::Regex = fields_valid::Regex::new(#rex).unwrap();)*
            }
            impl #impl_generics fields_valid::FieldsValidate for #struct_ident #type_generics #where_clause {
                #[inline]
                fn fields_validate(&self) -> std::result::Result<(), &'static str> {
                    #ts
                    std::result::Result::Ok(())
                }
            }
        }
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
