use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, ItemFn, LitBool, LitStr, Token,
};

pub const DEFAULT_DATADOG_AGENT_ENDPOINT: &str = "http://localhost:8126";

struct DatadogArgs {
    endpoint: Option<String>,
    service_name: String,
    location: Option<bool>,
}

impl Parse for DatadogArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut endpoint = None;
        let mut service_name = None;
        let mut location = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            match ident.to_string().as_str() {
                "endpoint" => {
                    if let Ok(lit_str) = input.parse::<LitStr>() {
                        endpoint = Some(lit_str.value());
                    }
                }
                "service_name" => {
                    if let Ok(lit_str) = input.parse::<LitStr>() {
                        service_name = Some(lit_str.value());
                    }
                }
                "location" => {
                    if let Ok(lit_bool) = input.parse::<LitBool>() {
                        location = Some(lit_bool.value());
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "Unexpected argument",
                    ))
                }
            }

            if !input.is_empty() {
                let _comma: Option<Token![,]> = input.parse()?;
            }
        }

        // Ensure service_name was provided
        let service_name = service_name.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "`service_name` is required for `datadog` attribute",
            )
        })?;

        Ok(DatadogArgs {
            endpoint,
            service_name,
            location,
        })
    }
}

pub fn datadog(attr: TokenStream, item: TokenStream) -> TokenStream {
    let datadog_args = parse_macro_input!(attr as DatadogArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let endpoint = datadog_args.endpoint.as_deref();
    let service_name = datadog_args.service_name.as_str();
    let location = datadog_args.location.unwrap_or(false);

    let expanded = quote! {
        #input_fn {
            let _shutdown_handle = ::telemetry_batteries::tracing::datadog::DatadogBattery::init(#endpoint, #service_name, None, #location);
        }
    };

    TokenStream::from(expanded)
}
