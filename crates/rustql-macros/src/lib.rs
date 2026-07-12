use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro]
pub fn rustql(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        {
            let mut executor = rustql_core::executor::Executor::new(
                rustql_core::schema::Schema::new()
            );
            executor
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn resolver(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let fn_body = &item_fn.block;
    let attr_str = attr.to_string();

    let parts: Vec<&str> = attr_str.split('.').collect();

    let expanded = if parts.len() == 2 {
        let type_name = parts[0].trim();
        let field_name = parts[1].trim();
        let resolver_key = format!("{}.{}", type_name, field_name);

        quote! {
            executor.add_resolver(#resolver_key, |field, ctx| {
                #fn_body
            });
        }
    } else {
        quote! {
            #item_fn
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn schema(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        rustql_core::schema::Schema::new()
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros_exist() {
        assert!(true);
    }
}