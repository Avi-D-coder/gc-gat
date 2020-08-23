#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(specialization)]
#![feature(marker_trait_attr)]
#![feature(dropck_eyepatch)]

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::cell::UnsafeCell;
    use std::{mem, ops::Deref};

    pub trait Id {
        type T;
    }

    impl<T> Id for T {
        type T = T;
    }

    #[marker]
    pub trait TyEq<A> {}

    // impl<'a, A, B: CoerceLifetime> TyEq<A> for B where A: Id<T = B::Type<'a>> {}
    // impl<A> TyEq<A> for A {}

    // pub unsafe trait CoerceLifetime<'r> {
    //     type Type<#[may_dangle] 'l: 'r>: 'l;
    //     unsafe fn coerce_lifetime<'o, 'n>(old: &'o Self::Type<'o>) -> &'n Self::Type<'n> {
    //         mem::transmute(old)
    //     }
    // }

    // unsafe impl<#[may_dangle] 'r, T: CoerceLifetime<'r>> CoerceLifetime<'r> for Gc<'r, T> {
    //     type Type<#[may_dangle] 'l: 'r> = Gc<'l, T::Type<'l>>;
    // }

    // unsafe impl<#[may_dangle] 'r, T: 'static> CoerceLifetime<'r> for T {
    //     default type Type<#[may_dangle] 'l: 'r> = T;
    // }

    pub unsafe trait CoerceLifetime<'r> {
        type Type<'l> where 'r: 'l;
        unsafe fn coerce_lifetime<'o, 'n>(old: &'o Self::Type<'o>) -> &'n Self::Type<'n> {
            mem::transmute(old)
        }
    }

    unsafe impl<'r, T: CoerceLifetime<'r>> CoerceLifetime<'r> for Gc<'r, T> {
        type Type<'l> where 'r: 'l = Gc<'l, T::Type<'l>>;
    }

    unsafe impl<'r, T: 'static> CoerceLifetime<'r> for T {
        default type Type<'l> where 'r: 'l = T;
    }

    pub struct Arena<T> {
        vec: UnsafeCell<Vec<T>>,
    }

    #[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct Gc<'r, T>(&'r T, ());
    impl<'r, T> Copy for Gc<'r, T> {}
    impl<'r, T> Clone for Gc<'r, T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<'r, T> Deref for Gc<'r, T> {
        type Target = T;
        fn deref(&self) -> &T {
            self.0
        }
    }

    impl<'l, A: CoerceLifetime<'l>> Arena<A> {
        fn new() -> Self {
            Self {
                vec: UnsafeCell::new(Vec::new()),
            }
        }

        pub fn gc<'r, 'a: 'r, T>(&'a self, t: T) -> Gc<'r, T>
        where
            // A: TyEq<A::Type<'r>>,
        {
            todo!()
        }
    }
    impl<T> Drop for Arena<T> {
        fn drop(&mut self) {}
    }

    #[test]
    fn use_after_free_test() {
        struct Foo<'r>(Gc<'r, usize>);
        unsafe impl<'r> CoerceLifetime<'r> for Foo<'r> {
            type Type<'l> = Foo<'l>;
        }

        // let usizes: Arena<usize> = Arena::new();
        // let foos: Arena<Foo> = Arena::new();

        // let n = usizes.gc(1usize);
        // let foo = foos.gc(Foo(n));
    }

    // fn foo<'r>(n: usize, usizes: &'r Arena<usize>) -> Foo<'r> {
    //     let n = usizes.gc(n);
    //     Foo(n)
    // }

    #[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    enum List<'r, T: Copy> {
        Cons(T, Gc<'r, List<'r, T>>),
        Empty,
    }

    unsafe impl<'r, T: 'r + Copy> CoerceLifetime<'r> for List<'r, T> {
        type Type<'l> where 'r: 'l = List<'l, T>;
    }

    impl<'r, T: Copy> List<'r, T> {
        fn cons(
            head: T,
            tail: Gc<'r, List<'r, T>>,
            arena: &'r Arena<List<'static, T>>,
        ) -> Gc<'r, List<'r, T>> {
            arena.gc(List::Cons(head, tail))
        }
    }

    #[test]
    fn gc_alloc_test() {
        let a: Arena<usize> = Arena::new();
        let one: usize = *a.gc(1);
        // let one = *a.gc("foo");
    }

    // let lists: Arena<List<u8>> = Arena::new();
    // let lists: &Arena<List<u8>> = &lists;
    // List::cons(1, lists.gc(List::Empty), &lists);
    // lists.gc(List::Cons(1, lists.gc(List::Empty)));

    // let nodes: Arena<Node<u8, u8>> = Arena::new();
    // let nodes: &Arena<Node<u8, u8>> = nodes;

    // Map::default().insert(1, 1, &nodes);
}
