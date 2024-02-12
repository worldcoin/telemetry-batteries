use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Ident, ItemFn, LitBool, LitStr, Token,
};

pub const DEFAULT_HOST_ENDPOINT: &str = "localhost";
pub const DEFAULT_HOST_PORT: u16 = 8125;
pub const DEFAULT_BUFFER_SIZE: usize = 256;
pub const DEFAULT_QUEUE_SIZE: usize = 5000;

struct StatsdArgs {
    host: Option<String>,
    port: Option<u16>,
    queue_size: Option<usize>,
    buffer_size: Option<usize>,
    prefix: Option<String>,
}

impl Parse for StatsdArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut host = None;
        let mut port = None;
        let mut queue_size = None;
        let mut buffer_size = None;
        let mut prefix = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            match ident.to_string().as_str() {
                "host" => {
                    if let Ok(lit_str) = input.parse::<LitStr>() {
                        host = Some(lit_str.value());
                    }
                }
                "port" => {
                    if let Ok(lit_int) = input.parse::<syn::LitInt>() {
                        port = Some(lit_int.base10_parse::<u16>()?);
                    }
                }
                "queue_size" => {
                    if let Ok(lit_int) = input.parse::<syn::LitInt>() {
                        queue_size = Some(lit_int.base10_parse::<usize>()?);
                    }
                }
                "buffer_size" => {
                    if let Ok(lit_int) = input.parse::<syn::LitInt>() {
                        buffer_size = Some(lit_int.base10_parse::<usize>()?);
                    }
                }
                "prefix" => {
                    if let Ok(lit_str) = input.parse::<LitStr>() {
                        prefix = Some(lit_str.value());
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
                let _: Option<Token![,]> = input.parse()?;
            }
        }

        Ok(StatsdArgs {
            host,
            port,
            queue_size,
            buffer_size,
            prefix,
        })
    }
}

pub fn statsd(attr: TokenStream, item: TokenStream) -> TokenStream {
    let statsd_args = parse_macro_input!(attr as StatsdArgs);
    let mut input_fn = parse_macro_input!(item as ItemFn);

    // Use provided values or defaults
    let host = statsd_args
        .host
        .unwrap_or_else(|| DEFAULT_HOST_ENDPOINT.to_string());
    let port = statsd_args.port.unwrap_or(DEFAULT_HOST_PORT);
    let queue_size = statsd_args.queue_size.unwrap_or(DEFAULT_QUEUE_SIZE);
    let buffer_size = statsd_args.buffer_size.unwrap_or(DEFAULT_BUFFER_SIZE);
    let prefix = statsd_args.prefix.unwrap_or_default();

    let input_block = &input_fn.block;
    let new_block: syn::Block = parse_quote!({
        let host = #host;
        let prefix = #prefix;
        telemetry_batteries::metrics::statsd::StatsdBattery::init(
            &host,
            #port,
            #queue_size,
            #buffer_size,
            Some(&prefix),
        )?;

        #input_block
    });

    input_fn.block = Box::new(new_block);

    let expanded = quote! {
        #input_fn
    };

    TokenStream::from(expanded)
}
