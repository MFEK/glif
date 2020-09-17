// $v of type RefCell<State<T>> (the global state)
// Returns type glifparser::Glif
#[macro_export]
macro_rules! get_outline_mut {
    ($v:ident) => {
        $v.borrow_mut()
            .glyph
            .as_mut()
            .unwrap()
            .glif
            .outline
            .as_mut()
            .unwrap()
    };
}

#[macro_export]
macro_rules! get_outline {
    ($v:ident) => {
        $v.borrow()
            .glyph
            .as_ref()
            .unwrap()
            .glif
            .outline
            .as_ref()
            .unwrap()
    };
}
