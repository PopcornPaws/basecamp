use quote::ToTokens;
use syn::{
    Attribute, Expr, GenericArgument, Ident, PathArguments, PathSegment, Type, TypePath,
};

pub fn should_flatten(attributes: &[Attribute]) -> bool {
    extract_attribute_value(attributes, "record", "flatten").is_some()
}

//pub fn has_attribute(meta: Meta, ident: &str) -> bool {
//    if let Meta::List(meta_list) = meta {
//        for nested_meta in meta_list.nested {
//            if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
//                return path.is_ident(ident);
//            }
//        }
//    }
//    false
//}

pub fn first_path_segment(type_path: &TypePath) -> Option<&PathSegment> {
    type_path.path.segments.first()
}

pub fn inner_vec_type(type_path: &TypePath) -> Option<&Ident> {
    first_path_segment(type_path)
        .filter(|segment| segment.ident == "Vec")
        .map(|segment| &segment.arguments)
        .and_then(|arg| {
            if let PathArguments::AngleBracketed(data) = arg {
                data.args.first()
            } else {
                None
            }
        })
        .and_then(|arg| {
            if let GenericArgument::Type(Type::Path(path)) = arg {
                Some(path)
            } else {
                None
            }
        })
        .and_then(first_path_segment)
        .map(|segment| &segment.ident)
}

pub fn batch_ident(ident: &Ident) -> Ident {
    Ident::new(&format!("Batch{ident}"), ident.span())
}

pub fn find_attribute<'a>(attributes: &'a [Attribute], name: &'a str) -> Option<&'a Attribute> {
    attributes.iter().find(|attr| attr.path().is_ident(name))
}

pub fn extract_attribute_value(attributes: &[Attribute], name: &str, key: &str) -> Option<String> {
    let mut value_string = None;
    find_attribute(attributes, name).map(|attr| {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(key) {
                if let Ok(value) = meta.value() {
                    let expr: Expr = value.parse().unwrap();
                    value_string = Some(expr.to_token_stream().to_string());
                } else {
                    value_string = Some(String::new());
                }
            }
            Ok(())
        })
    });
    value_string
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::{parse_quote, ItemStruct};

    #[test]
    fn struct_level_attribute() {
        let input: ItemStruct = parse_quote! {
            #[record(table = mytable)]
            pub struct MyStruct;
        };

        let extracted = extract_attribute_value(&input.attrs, "record", "table");
        assert_eq!(extracted, Some("mytable".into()));

        let input: ItemStruct = parse_quote! {
            #[foo(bar)]
            pub struct MyStruct;
        };

        let extracted = extract_attribute_value(&input.attrs, "foo", "bar");
        assert_eq!(extracted, Some("".into()));

        let extracted = extract_attribute_value(&input.attrs, "baz", "bar");
        assert!(extracted.is_none());

        let extracted = extract_attribute_value(&input.attrs, "foo", "baz");
        assert!(extracted.is_none());
    }
}
