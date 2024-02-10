use proc_macro::TokenStream;

mod tracing;

#[proc_macro_attribute]
pub fn datadog(attr: TokenStream, item: TokenStream) -> TokenStream {
    tracing::datadog::datadog(attr, item)
}
