use std::ops::Range;
use syn::{NestedMeta::Lit, Lit::Int, Lit::Float, Lit::Str, NestedMeta};
use quote::{quote};
use syn::spanned::Spanned;

#[derive(Debug)]
pub enum ValidateRule {
    Len(Range<usize>),
    Range(Range<f64>),
    Regex(String),
    Eq(String),
    Email,
}

type ValidateNested = syn::punctuated::Punctuated<syn::NestedMeta, syn::Token!(,)>;

fn lit_to_float(meta: &NestedMeta) -> Option<f64> {
    match meta {
        Lit(Int(n)) => n.base10_parse::<f64>().ok(),
        Lit(Float(n)) => n.base10_parse::<f64>().ok(),
        _ => None
    }
}

impl ValidateRule {
    pub fn from_meta(meta: syn::Meta) -> syn::Result<Self> {
        match meta {
            syn::Meta::List(
                syn::MetaList {
                    path,
                    nested,
                    ..
                }
            ) => {
                if let Some(seg) = path.segments.first() {
                    match seg.ident.to_string().as_str() {
                        "len" => Self::from_len(nested),
                        "regex" => Self::from_regex(nested),
                        "range" => Self::from_range(nested),
                        "eq" => Self::from_eq(nested),
                        _ => Err(syn::Error::new_spanned(seg, "参数错误！暂无该方法"))
                    }
                } else {
                    Err(syn::Error::new_spanned(path, "参数错误！暂无该方法"))
                }
            }
            syn::Meta::Path(
                syn::Path {
                    segments,
                    ..
                }
            ) => {
                if let Some(seg) = segments.first() {
                    match seg.ident.to_string().as_str() {
                        "email" => Ok(Self::Email),
                        _ => Err(syn::Error::new_spanned(seg, "参数错误！暂无该方法"))
                    }
                } else {
                    Err(syn::Error::new_spanned(segments, "参数错误！暂无该方法"))
                }
            },
            _ => Err(syn::Error::new_spanned(meta, "参数错误！"))
        }
    }

    fn from_len(nested: ValidateNested) -> syn::Result<Self> {
        let one = if let Some(Lit(Int(one))) = nested.first() {
            one.to_string().parse::<usize>().expect("数值范围出错")
        } else {
            return Err(syn::Error::new_spanned(nested, "参数错误！"))
        };
        if let Some(Lit(Int(two))) = nested.last() {
            let two = two.to_string().parse::<usize>().expect("数值范围出错");
            let min = one.min(two);
            let max = one.max(two);
            Ok(Self::Len(min..max))
        } else {
            Ok(Self::Len(one..one))
        }
    }

    fn from_regex(nested: ValidateNested) -> syn::Result<Self> {
        if let Some(Lit(Str(rx))) = nested.first() {
            Ok(Self::Regex(rx.value()))
        } else {
            Err(syn::Error::new_spanned(nested, "参数错误！"))
        }
    }

    fn from_range(nested: ValidateNested) -> syn::Result<Self> {
        let first = if let Some(first) = nested.first() {
            first
        } else {
            return Err(syn::Error::new_spanned(nested, "参数错误！"))
        };
        let one = lit_to_float(first).unwrap();
        if let Some(last) = nested.last() {
            let two = lit_to_float(last).unwrap();
            let min = one.min(two);
            let max = one.max(two);
            Ok(Self::Range(min..max))
        } else {
            Ok(Self::Range(one..one))
        }
    }

    fn from_eq(nested: ValidateNested) -> syn::Result<Self> {
        if let Some(Lit(Str(eq))) = nested.first() {
            Ok(Self::Eq(eq.value()))
        } else {
            Err(syn::Error::new_spanned(nested, "参数错误！"))
        }
    }
}

#[derive(Debug)]
pub struct ValidateMeta<'v> {
    validates: Vec<ValidateRule>,
    pub err_msg: String,
    pub struct_ident: &'v syn::Ident,
}

impl<'v> ValidateMeta<'v> {
    pub fn from_valid_attr(attr: &syn::Attribute, struct_ident: &'v syn::Ident) -> syn::Result<Self> {
        let meta = match attr.parse_meta() {
            Ok(meta) => {
                // eprintln!("meta = {:#?}", meta);
                meta
            },
            Err(err) => {
                // eprintln!("attr = {:#?}", attr);
                return Err(err)
            }
        };
        let nested = if let syn::Meta::List(
            syn::MetaList {
                path,
                nested,
                ..
            }
        ) = meta {
            match path.segments.first() {
                Some(seg) if seg.ident.to_string() == "valid" => {
                    nested
                },
                _ => return Err(syn::Error::new_spanned(path, "无法处理到该属性值"))
            }
        } else {
            return Err(syn::Error::new_spanned(meta, "语法错误！"))
        };
        let mut validates = Vec::new();
        let mut err_msg = None;
        for nm in nested {
            match nm {
                syn::NestedMeta::Meta(meta) => {
                    validates.push(ValidateRule::from_meta(meta)?)
                }
                syn::NestedMeta::Lit(syn::Lit::Str(msg)) => {
                    err_msg = Some(msg.value())
                },
                _ => {}
            }
        }
        Ok(Self {
            struct_ident,
            validates,
            err_msg: err_msg.unwrap_or("参数错误！".to_string())
        })
    }

    pub fn validate_token(
        &self,
        o_field_name: &Option<syn::Ident>,
        is_optional: bool,
        type_name: &str,
        static_ref: (&mut Vec<syn::Ident>, &mut Vec<String>)
    ) -> proc_macro2::TokenStream {
        let mut final_tokenstream = proc_macro2::TokenStream::new();
        let mut field_name = proc_macro2::TokenStream::new();
        field_name.extend(if is_optional {quote!(#o_field_name.as_ref().unwrap())} else {quote!(#o_field_name)});
        for (index, validate) in self.validates.iter().enumerate() {
            let t = match validate {
                ValidateRule::Regex(rx) => {
                    let rex_name = format!(
                        "{}_{}_REGEX",
                        self.struct_ident.to_string().to_uppercase(),
                        o_field_name.as_ref().unwrap().to_string().to_uppercase()
                    );
                    let rex_name_ident = syn::Ident::new(&rex_name, o_field_name.span());
                    let qt = quote!(!#rex_name_ident.is_match(&self.#field_name));
                    static_ref.0.push(rex_name_ident);
                    static_ref.1.push(rx.clone());
                    qt
                    // quote!(!fields_valid::validates::match_regex(#rx, &self.#field_name))
                }
                ValidateRule::Len(range) => {
                    let min = range.start;
                    let max = range.end;
                    quote!{!(#min..#max).contains(&self.#field_name.chars().count())}
                }
                ValidateRule::Email => {
                    quote!(!fields_valid::validates::email(&self.#field_name))
                }
                ValidateRule::Range(Range{ start, end}) => {
                    if type_name == "BigDecimal" {
                        quote!{!(#start..#end).contains(&self.#field_name.to_f64().unwrap())}
                    } else {
                        quote!{!(#start..#end).contains(&(self.#field_name as f64))}
                    }
                }
                ValidateRule::Eq(eq) => {
                    if eq.chars().next().unwrap_or('\0') == '#' {
                        let other_field = syn::Ident::new(&eq[1..], o_field_name.span());
                        quote!{self.#field_name != self.#other_field}
                    } else {
                        quote!{self.#field_name != #eq}
                    }
                }
            };
            if index == 0 {
                final_tokenstream.extend(t)
            } else {
                final_tokenstream.extend(quote!(|| #t))
            }
        }
        if is_optional {
            quote! {self.#o_field_name.is_some() && (#final_tokenstream)}
        } else {
            final_tokenstream
        }
    }
}
