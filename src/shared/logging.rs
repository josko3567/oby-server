#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        let pure = name.strip_suffix("::f").unwrap();
        let split = pure.clone().split_once(" ");
        if split.is_some() {
            split.unwrap().0
        } else {
            pure
        }
    }}
}

macro_rules! logf {
    ($fmt:expr) => {
        format!("({}) {}", crate::function!(), $fmt)
    }
}
pub(crate) use logf;
