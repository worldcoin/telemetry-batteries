fn main() {
    // Ensure a TLS backend is enabled
    let rustls = cfg!(feature = "rustls");
    let native_tls = cfg!(feature = "native-tls");

    if !rustls && !native_tls {
        panic!(
            "At least one TLS feature must be enabled: 'rustls' or 'native-tls'"
        );
    }
}
