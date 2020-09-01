#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(specialization)]
#![feature(negative_impls)]
#![feature(optin_builtin_traits)]

use auto_traits::Immutable;

fn main() {
    println!("Hello, world!");
}

pub unsafe trait Life: Immutable {
    type L<'l>: 'l + Life + Immutable;
}

unsafe impl<T: 'static + Immutable> Life for T {
    default type L<'l> = T;
}

pub struct Gc<'r, T: Life>(&'r T);
unsafe impl<'r, T: Life> Life for Gc<'r, T> {
    type L<'l> = Gc<'l, T::L<'l>>;
}

impl<'r, T: Life> Copy for Gc<'r, T> {}
impl<'r, T: Life> Clone for Gc<'r, T> {
    fn clone(&self) -> Self {
        *self
    }
}

unsafe impl<'r, T: Life> Life for Option<T> {
    type L<'l> = Option<T::L<'l>>;
}

mod auto_traits {
    use super::Gc;
    use crate::Life;
    use std::cell::UnsafeCell;

    pub unsafe auto trait NoGc {}
    impl<'r, T> !NoGc for Gc<'r, T> {}
    // unsafe impl<'r, T: NoGc> NoGc for Box<T> {}

    pub trait HasGc {
        const HAS_GC: bool;
    }

    impl<T> HasGc for T {
        default const HAS_GC: bool = true;
    }

    impl<T: NoGc> HasGc for T {
        const HAS_GC: bool = false;
    }

    /// Shallow immutability
    pub unsafe auto trait Immutable {}
    impl<T> !Immutable for &mut T {}
    impl<'r, T> !Immutable for &'r T {}
    impl<T> !Immutable for UnsafeCell<T> {}
    unsafe impl<T> Immutable for Box<T> {}
    unsafe impl<'r, T: Life> Immutable for Gc<'r, T> {}

    /// Should be implemented with each `Trace` impl.
    pub auto trait NotDerived {}
    impl<'l, T> !NotDerived for Gc<'l, T> {}
}

#[derive(Copy, Clone)]
pub struct List<'r, T: Life>(Option<Gc<'r, Elem<'r, T>>>);

#[derive(Clone)]
pub struct Elem<'r, T: Life> {
    next: List<'r, T>,
    value: T,
}

impl<'r, T: Life + Copy> Copy for Elem<'r, T> {}
unsafe impl<'r, T: Life> Life for List<'r, T> {
    type L<'l> = List<'l, T::L<'l>>;
}
unsafe impl<'r, T: Life> Life for Elem<'r, T> {
    type L<'l> = Elem<'l, T::L<'l>>;
}

#[test]
fn test() {
    #![allow(unreachable_code)]
    let _: List<List<usize>> = todo!();
    let _: List<List<Gc<String>>> = todo!();
    let _: List<Gc<String>> = todo!();
    let _: List<Gc<String>> = todo!();
    // let _: List<List<Gc<&usize>>> = todo!(); //~ Err the trait bound `&usize: auto_traits::Immutable` is not satisfied
}
