#[macro_export]
macro_rules! stub {
    () => {
        todo!("{} is not implemented", "stub")
    };
}
