#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(specialization)]
#![feature(negative_impls)]
#![feature(optin_builtin_traits)]
#![feature(marker_trait_attr)]
#![feature(dropck_eyepatch)]
#![feature(const_generics)]
#![allow(unreachable_code)]

use auto_traits::{HasGc, NoGc, NotDerived};
use std::ops::Deref;

fn main() {
    println!("Hello, world!");
}

pub unsafe trait Life {
    type L<'l>: 'l + Life<L<'r> = Self::L<'r>>;
}

#[test]
fn usize_test() {
    let a: <usize as Life>::L<'static> = 1usize;
}

unsafe impl<T: 'static + NoGc> Life for T {
    type L<'l> = T;
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct Gc<'r, T>(&'r T);

impl<'r, T> Deref for Gc<'r, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
unsafe impl<'r, T: Life> Life for Gc<'r, T> {
    type L<'l> = Gc<'l, T::L<'l>>;
}
impl<'r, T: Life> Copy for Gc<'r, T> {}
impl<'r, T: Life> Clone for Gc<'r, T> {
    fn clone(&self) -> Self {
        *self
    }
}

type Of<'a, #[may_dangle] T> = <T as Life>::L<'a>;

#[test]
fn eq_test() {
    fn eq_usize<'a, 'b>(a: <usize as Life>::L<'a>, b: <usize as Life>::L<'b>) {
        a == b;
    }

    fn eq_str<'a, 'b>(a: <&'static str as Life>::L<'a>, b: <&'static str as Life>::L<'b>) {
        a == b;
    }

    fn eq_t<'a, 'b, T: Life>(a: T::L<'a>, b: T::L<'b>) -> bool
    where
        T::L<'a>: Eq,
        'a: 'b,
        'b: 'a,
    {
        a == b //~ [rustc E0623] [E] lifetime mismatch ...but data from `a` flows into `b` here
    }

    fn use_eq_t_usize() {
        let arena: Arena<usize> = Arena::new();

        let a: Gc<usize> = arena.gc(usize::default());
        let b: Gc<usize> = arena.gc(usize::default());

        eq_t::<Gc<usize>>(a, b);

        let arena_: Arena<usize> = Arena::new();

        let a_ = arena_.mark(a);
        let b_ = arena_.mark(b);

        drop(arena);

        eq_t::<Gc<usize>>(a_, b_);
    }

    fn use_eq_t_t<T: Life>()
    where
        for<'a> Of<'a, T>: Default + Eq,
    {
        let arena: Arena<T> = Arena::new();

        let a: Gc<Of<T>> = arena.gc(Of::<T>::default());
        let b: Gc<Of<T>> = arena.gc(Of::<T>::default());

        eq_t::<Gc<T>>(a, b);

        let arena_: Arena<T> = Arena::new();

        let a_: Gc<Of<T>> = arena_.mark(a);
        // let b_: Gc<T> = arena_.mark(b); //~ [rustc E0308] [E] mismatched types expected struct `Gc<'_, T>` found struct `Gc<'_, <T as Life>::L<'_>>`

        drop(arena);

        // eq_t(a_, b_);
    }

    // fn eq_alloc<'a, 'b, T: Life + Eq>(arena: &'a Arena<T>, a: T, b: &T::L<'b>) -> bool
    // where
    //     T::L<'a>: Eq,
    //     'a: 'b,
    //     'b: 'a
    // {
    //     let a = arena.gc(a);
    //     *a == b //~ [rustc E0623] [E] lifetime mismatch ...but data from `a` flows into `b` here
    // }
}

pub unsafe trait ID<T> {}
unsafe impl<T> ID<T> for T {}

pub struct Arena<#[may_dangle] A>(Vec<A>);

impl<#[may_dangle] T: Life> Arena<T> {
    pub fn gc<'r>(&'r self, t: T::L<'_>) -> Gc<'r, T::L<'r>> {
        todo!()
    }

    pub fn new() -> Arena<T> {
        todo!()
    }

    pub fn mark<'n>(&'n self, o: Gc<'_, T::L<'_>>) -> Gc<'n, T::L<'n>> {
        unsafe { std::mem::transmute(o) }
    }
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

// #[cfg(off)]
mod list {
    use super::*;
    #[derive(Copy, Clone)]
    pub struct List<'r, T: 'r + Life>(Option<Gc<'r, Elem<'r, T>>>);

    #[derive(Clone)]
    pub struct Elem<'r, T: 'r + Life> {
        next: List<'r, T>,
        value: T,
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

    impl<'r, T: Life + Clone + Life<L = T>> List<'r, T> {
        /// Prepend `value` to a list.
        /// The arguments are in reverse order.
        pub fn cons<'a: 'r>(self, value: T, arena: &'a Arena<Elem<T>>) -> List<'r, T::L<'a>> {
            List::from(arena.gc(Elem { next: self, value }))
        }
    }

    #[test]
    fn test() {
        #![allow(unreachable_code)]
        // let _: List<List<usize>> = todo!();
        let _: List<List<Gc<String>>> = todo!();
        let _: List<Gc<String>> = todo!();
        let _: List<Gc<String>> = todo!();

        // let _: usize = Elem::<usize> {
        //     next: List(None),
        //     value: 1,
        // }
        // .value;

        let arena: Arena<Elem<usize>> = todo!();

        let _: usize = List::<usize>::from(arena.gc(Elem {
            next: List(None),
            value: 1usize,
        }))
        .0
        .unwrap()
        .0
        .value;

        // fn foo<'r, T: Live<'r>>(ll: &Arena<Elem<List<T>>>, lt: &Arena<Elem<T>>, value: T) {
        //     let val: List<T::R> = List::from(Elem::gc(lt, List(None), value));
        //     let _: Gc<Elem<List<T::R>>> = Elem::gc(ll, List(None), val);
        // }
        // let _: List<List<Gc<&usize>>> = todo!(); //~ Err the trait bound `&usize: auto_traits::Immutable` is not satisfied
    }
}
