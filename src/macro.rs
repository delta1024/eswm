extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn rule(item: TokenStream) -> TokenStream {
    let item = item.to_string();
    let item: Vec<&str> = item.as_str().split(',').collect();
    let (good_string, _) = item[3].split_at(item[3].len() - 1);
    let formatted = format!(
        "ParseRule {{ prefix: {}, infix: {}, precedence: {}}}",
        item[1], item[2], good_string
    );
    formatted.parse().unwrap()
}
