#[macro_export]
macro_rules! fn_ensure_option {
    ([$($fn:tt)*], $($target:ident).*, if_none: $none:literal, if_some: $some:literal,) => {
        $($fn)* (&self, expects: Option<()>) -> Result<(), String> {
            match expects {
                Some(_) if self.$($target.)*is_none() => Err(String::from($none)),
                None if self.$($target.)*is_some() => Err(String::from($some)),
                _ => Ok(()),
            }
        }
    };
}
