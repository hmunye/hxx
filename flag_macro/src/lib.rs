use proc_macro::*;

#[proc_macro_attribute]
pub fn flag(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut args_iter = args.into_iter();

    while let Some(arg) = args_iter.next() {
        println!("arg = {arg}");
    }

    input
}
