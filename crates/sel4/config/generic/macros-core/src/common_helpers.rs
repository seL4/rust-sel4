macro_rules! parse_or_return {
    ($tokenstream:ident as $ty:ty) => {
        match parse2::<$ty>($tokenstream) {
            Ok(parsed) => parsed,
            Err(err) => {
                return err.to_compile_error();
            }
        }
    };
}

pub(crate) use parse_or_return;
