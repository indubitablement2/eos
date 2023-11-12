pub fn runtime() -> &'static tokio::runtime::Runtime {
    unsafe { _RUNTIME.as_ref().unwrap_unchecked() }
}
pub fn _init_runtime() {
    unsafe {
        _RUNTIME = Some(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
    }
}
static mut _RUNTIME: Option<tokio::runtime::Runtime> = None;
