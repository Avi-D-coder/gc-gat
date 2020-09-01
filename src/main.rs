#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(specialization)]
#![feature(negative_impls)]
#![feature(optin_builtin_traits)]
#![feature(marker_trait_attr)]

use auto_traits::{Immutable, NotDerived};

fn main() {
    println!("Hello, world!");
}

pub unsafe trait Life: Immutable {
    type L<'l>: 'l + Life + Immutable;
}

pub struct IncorrectImpls;

unsafe impl<T: Immutable> Life for T {
    default type L<'l> = IncorrectImpls;
}

unsafe impl<T: 'static + Immutable + NotDerived> Life for T {
    type L<'l> = T;
}

pub trait GC<T>: Life {}
impl<'r, T: Life, S: Life + TyEq<T>> GC<T> for S {}

#[marker]
pub unsafe trait TyEq<B> {}
unsafe impl<T> TyEq<T> for T {}
unsafe impl<'l, T: Life> TyEq<T> for T::L<'l> {}
unsafe impl<'l, T: Life> TyEq<T::L<'l>> for T {}
unsafe impl<'l, A: Life, B: Life> TyEq<B> for A where A::L<'l>: ID<B::L<'l>> {}

pub unsafe trait ID<T> {}
unsafe impl<T> ID<T> for T {}

pub struct Arena<A>(Vec<A>);

pub struct Gc<'r, T: 'r + Life>(&'r T);
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

impl<T> !NotDerived for Option<T> {}

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

mod list {
    use super::*;
    #[derive(Copy, Clone)]
    pub struct List<'r, T: 'r + Life>(Option<Gc<'r, Elem<'r, T>>>);

    #[derive(Clone)]
    pub struct Elem<'r, T: 'r + Life> {
        next: List<'r, T>,
        value: T::L<'r>,
    }

    impl<'r, T: 'r + Life> Elem<'r, T> {
        pub fn gc<'a: 'r>(
            arena: &'a Arena<Elem<T>>,
            next: List<T>,
            value: T,
        ) -> Gc<'r, Elem<'r, T::L<'r>>> {
            todo!()
        }
    }

    impl<'r, T: 'r + Life + Copy> Copy for Elem<'r, T> where T::L<'r>: Copy {}
    unsafe impl<'r, T: 'r + Life> Life for List<'r, T> {
        type L<'l> = List<'l, T::L<'l>>;
    }
    unsafe impl<'r, T: 'r + Life> Life for Elem<'r, T> {
        type L<'l> = Elem<'l, T::L<'l>>;
    }

    impl<'r, T: Life> From<Gc<'r, Elem<'r, T>>> for List<'r, T> {
        fn from(e: Gc<'r, Elem<'r, T>>) -> Self {
            List(Some(e))
        }
    }

    impl<'r, T: 'r + Life + Clone> List<'r, T> {
        /// Prepend `value` to a list.
        /// The arguments are in reverse order.
        pub fn cons<'a: 'r>(self, value: T, arena: &'a Arena<Elem<T>>) -> List<'r, T> {
            List::from(Elem::gc(arena, self, value))
        }
    }

    #[test]
    fn test() {
        #![allow(unreachable_code)]
        let _: List<List<usize>> = todo!();
        let _: List<List<Gc<String>>> = todo!();
        let _: List<Gc<String>> = todo!();
        let _: List<Gc<String>> = todo!();

        let _: usize = Elem::<usize> {
            next: List(None),
            value: 1,
        }
        .value;

        let _: usize = List::from(Elem::gc(todo!(), List(None), 1))
            .0
            .unwrap()
            .0
            .value;

        fn foo<T: Life>(arena: &Arena<Elem<T>>, value: T) {
            let _: T = Elem::<List<T>> {
                next: List(None),
                value: List::from(Elem::gc(arena, List(None), value)),
            }
            .value
            .0
            .unwrap()
            .0
            .value;
        }
        // let _: List<List<Gc<&usize>>> = todo!(); //~ Err the trait bound `&usize: auto_traits::Immutable` is not satisfied
    }
}
