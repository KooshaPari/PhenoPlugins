//! 5 smoke tests for the PluginLoader port.
use ports::adapters::native::NativeLoader;
use ports::adapters::wasm::WasmLoader;
use ports::plugin_loader::PluginLoader;

#[tokio::test]
async fn wasm_backend() {
    assert_eq!(WasmLoader.backend(), "wasm");
}

#[tokio::test]
async fn native_backend() {
    assert_eq!(NativeLoader.backend(), "native");
}

#[tokio::test]
async fn wasm_load_nonexistent_returns_err() {
    assert!(WasmLoader
        .load(std::path::Path::new("/nope.wasm"))
        .await
        .is_err());
}

#[tokio::test]
async fn native_load_nonexistent_returns_err() {
    assert!(NativeLoader
        .load(std::path::Path::new("/nope.so"))
        .await
        .is_err());
}

#[tokio::test]
async fn trait_object_safe() {
    let _t: Box<dyn PluginLoader> = Box::new(WasmLoader);
}
