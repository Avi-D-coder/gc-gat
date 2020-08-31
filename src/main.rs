#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(specialization)]
#![feature(marker_trait_attr)]
#![feature(dropck_eyepatch)]
#![feature(associated_type_defaults)]
#![feature(negative_impls)]
#![feature(optin_builtin_traits)]

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::cell::UnsafeCell;
    use std::{mem, ops::Deref};

    pub struct Map<'r, K, V>(Option<Gc<'r, Node<'r, K, V>>>)
    where
        K: 'r,
        V: 'r;

    pub struct Node<'r, K, V>
    where
        K: 'r,
        V: 'r,
    {
        key: K,
        size: usize,
        left: Map<'r, K, V>,
        right: Map<'r, K, V>,
        value: V,
    }

    // impl<'r, K: GC<T = K>, V: GC<T = V>> Node<'r, K, V> {
    //     pub fn gc<'new_root, 'arena: 'new_root, 'left, 'right>(
    //         key: K,
    //         size: usize,
    //         left: impl GC<T = Self>,
    //         right: impl GC<T = Self>,
    //         value: V,
    //         arena: &'arena Arena<Self>,
    //     ) -> Gc<'new_root, <Self as Life>::L<'new_root>> {
    //         unsafe {
    //             arena.gc(Node {
    //                 key: todo!(),
    //                 size,
    //                 left: todo!(),
    //                 right: todo!(),
    //                 value: todo!(),
    //             });
    //         }
    //     }
    // }

    unsafe impl<'r, K: Life +  GC<T = K>, V:  Life + GC<T = V>> Life for Node<'r, K, V> {
        type L<'l> = Node<'l, K::L<'l>, V::L<'l>>;
    }

    pub trait ID {
        type T;
    }
    impl<T> ID for T {
        type T = T;
    }
    pub trait TypeEq<A> {}
    // impl<A: CoerceLifetime, B: CoerceLifetime> TypeEq<A> for B where for<'a> A::Type<'a>: ID<B::Type<'a>>
    // {}
    // #[marker]
    // pub trait TyEq<A> {}

    // impl<'a, 'b, A, B: CoerceLifetime> TyEq<A> for B where A: Id<T = B::Type<'a>> {}
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

    // #[test]
    // fn function_name_test() {
    //     foo::<usize>(1usize);
    // }

    #[test]
    fn gc_test() {
        let a: Arena<usize> = Arena::new();
        let one: Gc<usize> = a.gc(1usize);
    }

    pub auto trait NotDerived {}
    impl<'l, T> !NotDerived for Gc<'l, T> {}

    pub trait GC: Life {
        type T: TyEq<B = Self::L<'static>>;
    }
    impl<'r, T: Life> GC for T {
        type T = T;
    }

    pub unsafe fn coerce_lifetime<'n, T: Sized + Life>(old: T) -> T::L<'n> {
        mem::transmute(old)
    }

    pub unsafe trait Life {
        type L<'l>: 'l + Life + GC<T = Self>;
    }

    // pub unsafe fn coerce_lifetime<'n, T>(old: T) -> T::L<'n> where T:Life + Sized, T::L<'n>: Life + Sized {
    //     mem::transmute(old)
    // }

    unsafe impl<'r, T: Life> Life for Gc<'r, T> {
        type L<'l> = Gc<'l, T::L<'l>>;
    }

    unsafe impl<T: 'static + NotDerived> Life for T {
        type L<'l> = T;
    }

    pub unsafe trait TyEq: Life {
        type B: Life where Self::B: ID<T = Self::L<'static>>;
    }
    unsafe impl<A: Life, B: Life> TyEq for A {
        type B = B;
    }

    // unsafe impl<T: 'static + Static<T>> Life for T {
    //     type Type<'l> = T;
    // }

    // pub trait Static<T: 'static>: Life<Type = T> {}

    //     pub unsafe trait CoerceLifetime {
    //         type Type<'l>: 'l;
    //         unsafe fn coerce_lifetime<'o, 'n>(old: &'o Self::Type<'o>) -> &'n Self::Type<'n> {
    //             mem::transmute(old)
    //         }
    //     }

    //     unsafe impl<'r, T: CoerceLifetime> CoerceLifetime for Gc<'r, T> {
    //         type Type<'l> = Gc<'l, T::Type<'l>>;
    //     }

    //     default unsafe impl<T: 'static> CoerceLifetime for T {
    //         type Type<'l> where T: 'static = T;
    //     }

    pub struct Arena<T> {
        vec: UnsafeCell<Vec<T>>,
    }

    //     #[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct Gc<'r, T>(&'r T, ());
    //     impl<'r, T> Copy for Gc<'r, T> {}
    //     impl<'r, T> Clone for Gc<'r, T> {
    //         fn clone(&self) -> Self {
    //             *self
    //         }
    //     }
    //     impl<'r, T> Deref for Gc<'r, T> {
    //         type Target = T;
    //         fn deref(&self) -> &T {
    //             self.0
    //         }
    //     }

    impl<'l, A: Life> Arena<A> {
        fn new() -> Self {
            Self {
                vec: UnsafeCell::new(Vec::new()),
            }
        }

        pub fn gc<'r, 'a: 'r, T: Life>(&'a self, t: T) -> Gc<'r, T::L<'r>>
        {
            todo!()
        }
    }
    //     impl<T> Drop for Arena<T> {
    //         fn drop(&mut self) {}
    //     }

    // #[test]
    // fn use_after_free_test() {
    //     struct Foo<'r>(Gc<'r, usize>);
    //     unsafe impl<'r> Life for Foo<'r> {
    //         type Type<'l> = Foo<'l>;
    //     }

    //     let usizes: Arena<usize> = Arena::new();
    //     let foos: Arena<Foo> = Arena::new();

    //     let n = usizes.gc(1usize);
    //     let foo = foos.gc(Foo(n));

    //     fn foo<'r>(n: usize, usizes: &'r Arena<usize>) -> Foo<'r> {
    //         let n = usizes.gc(n);
    //         Foo(n)
    //     }
    // }
    // #[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    // enum List<'r, T: Copy> {
    //     Cons(T, Gc<'r, List<'r, T>>),
    //     Empty,
    // }

    // unsafe impl<'r, T: 'r + Copy> CoerceLifetime<'r> for List<'r, T> {
    //     type Type<'l> where 'r: 'l = List<'l, T>;
    // }

    // impl<'r, T: Copy> List<'r, T> {
    //     fn cons(
    //         head: T,
    //         tail: Gc<'r, List<'r, T>>,
    //         arena: &'r Arena<List<T>>,
    //     ) -> Gc<'r, List<'r, T>> {
    //         arena.gc(List::Cons(head, tail))
    //     }
    // }

    // #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
    // pub struct List<'r, T: Copy>(Option<Gc<'r, Elem<'r, T>>>)
    // where
    //     T: 'r;

    // #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
    // pub struct Elem<'r, T: Copy>
    // where
    //     T: 'r,
    // {
    //     pub next: List<'r, T>,
    //     pub value: T,
    // }

    // impl<'r, T: Copy> From<Gc<'r, Elem<'r, T>>> for List<'r, T> {
    //     fn from(e: Gc<'r, Elem<'r, T>>) -> Self {
    //         List(Some(e))
    //     }
    // }

    // impl<'r, T: 'r + Copy> List<'r, T> where Self: 'r {
    //     /// Prepend `value` to a list.
    //     /// The arguments are in reverse order.
    //     pub fn cons<'a: 'r>(self, value: T, arena: &'a Arena<Elem<T>>) -> List<'r, T> {
    //         List::from(arena.gc(Elem { value, next: self }))
    //     }

    //     /// Inserts an element at position `index`.
    //     /// This is equivalent `Vec::insert` not Haskell's `insert :: Ord a => a -> [a] -> [a]`.
    //     ///
    //     /// # Panics
    //     ///
    //     /// Panics if `index > len`.
    //     /// This function is recursive and may cause a stack overflow.
    //     ///
    //     /// TODO Replace with non recursive variant.
    //     pub fn insert(self, index: usize, arena: &Arena<Elem<T>>) -> List<'r, T> {
    //         // self.iter().take(index).fold(List::default(), )
    //         let Gc(Elem { value, next }, _) = self.0.unwrap();
    //         List::from(arena.gc(Elem {
    //             value: value.clone(),
    //             next: next.insert(index - 1, arena),
    //         }))
    //     }
    // }

    #[test]
    fn gc_alloc_test() {
        // let a: Arena<usize> = Arena::new();
        // let one: usize = *a.gc(1);
        // let one = *a.gc("foo"); //~ Err
        // [rustc E0271] [E] type mismatch resolving `<<usize as tests::CoerceLifetime<'_>>::Type<'_> as tests::Id>::T == <&str as tests::CoerceLifetime<'_>>::Type<'_>`
        //         expected type `<usize as tests::CoerceLifetime<'_>>::Type<'_>`
        // found associated type `<&str as tests::CoerceLifetime<'_>>::Type<'_>`
        // required because of the requirements on the impl of `tests::TyEq<<usize as tests::CoerceLifetime<'_>>::Type<'_>>` for `&str`
    }

    // let lists: Arena<List<u8>> = Arena::new();
    // let lists: &Arena<List<u8>> = &lists;
    // List::cons(1, lists.gc(List::Empty), &lists);
    // lists.gc(List::Cons(1, lists.gc(List::Empty)));

    // let nodes: Arena<Node<u8, u8>> = Arena::new();
    // let nodes: &Arena<Node<u8, u8>> = nodes;

    // Map::default().insert(1, 1, &nodes);
}
