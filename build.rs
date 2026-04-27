fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let version = env!("CARGO_PKG_VERSION");
        let mut res = winresource::WindowsResource::new();
        res.set("FileDescription", &format!("ps-battery ({})", version));
        res.set("ProductName", "ps-battery");
        res.set("FileVersion", version);
        res.set("ProductVersion", version);
        res.compile().expect("failed to compile Windows resources");
    }
}
