#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(specialization)]
#![feature(marker_trait_attr)]
#![feature(associated_type_defaults)]
#![feature(negative_impls)]
#![feature(optin_builtin_traits)]

fn main() {
    println!("Hello, world!");
}

/// Should be implemented with each `Trace` impl.
pub auto trait NotDerived {}
impl<'l, T> !NotDerived for Gc<'l, T> {}

unsafe trait Life {
    type L<'l>: 'l + Life;
}

unsafe impl<'r, T: 'static + NotDerived> Life for T {
    type L<'l> = T;
}

struct Gc<'r, T>(&'r T);
unsafe impl<'r, T: Life> Life for Gc<'r, T> {
    type L<'l> = Gc<'l, T::L<'l>>;
}

unsafe impl<'r, T: Life> Life for Option<T> {
    type L<'l> = Option<T::L<'l>>;
}

impl<T> !NotDerived for Option<T> {}

unsafe impl<'r, T: Life> Life for List<'r, T> {
    type L<'l> = List<'l, T::L<'l>>;
}

unsafe impl<'r, T: Life> Life for Elem<'r, T> {
    type L<'l> = Elem<'l, T::L<'l>>;
}

impl<'r, T> !NotDerived for Elem<'r, T> {}
impl<'r, T> !NotDerived for List<'r, T> {}

struct List<'r, T: Life>(<Option<Gc<'r, Elem<'r, T::L<'r>>>> as Life>::L<'r>);
struct Elem<'r, T: Life> {
    next: List<'r, T::L<'r>>,
    value: T::L<'r>,
}
