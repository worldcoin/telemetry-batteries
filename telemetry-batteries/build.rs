fn main() {
    // Ensure exactly one TLS backend is enabled
    let rustls = cfg!(feature = "rustls");
    let native_tls = cfg!(feature = "native-tls");

    match (rustls, native_tls) {
        (false, false) => {
            panic!("At least one TLS feature must be enabled: 'rustls' or 'native-tls'");
        }
        (true, true) => {
            panic!("Only one TLS feature can be enabled at a time: 'rustls' or 'native-tls'");
        }
        _ => {}
    }
}
