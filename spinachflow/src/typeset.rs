

// trait Identity<T> {}
// impl<T> Identity<T> for T {}

// trait LinkedSetOrNone {}
// impl LinkedSetOrNone for () {}

// trait LinkedSet {
//     type Item;
//     type Next: LinkedSetOrNone;
// }
// impl<T: LinkedSet> LinkedSetOrNone for T {}

// impl<Item, Next: LinkedSetOrNone> LinkedSet for (Item, Next) {
//     type Item = Item;
//     type Next = Next;
// }

// trait Exclude<Item> {
//     type Output;
// }

// impl<ItemToRemove, Item, Next: Exclude<ItemToRemove>> Exclude<ItemToRemove> for (Item, Next) {
//     default type Output = &'static str; //(Item, <Next as Exclude<ItemToRemove>>::Output);
// }
// impl<ItemToRemove, Item, Next: Exclude<ItemToRemove>> Exclude<ItemToRemove> for (Item, Next)
// where
//     ItemToRemove: Identity<Item>,
// {
//     type Output = <Next as Exclude<ItemToRemove>>::Output;
// }
// impl<Item> Exclude<Item> for () {
//     type Output = ();
// }


// type ZZZ = (u8, (i16, (u16, (u32, ()))));
// type XYZ = <ZZZ as Exclude<i16>>::Output;

// fn _test() {
//     let z: <() as Exclude<i16>>::Output = ();
//     let z: <(i16, ()) as Exclude<i16>>::Output = ();
//     let z: <(u16, ()) as Exclude<i16>>::Output = "hello"; //(0_u16, ()); //(0_u16, ());
//     let xyz: XYZ = (0_u8, (0_u16, (0_u32, ())));
// }


// // pub trait Bool {}
// // pub enum True {}
// // impl Bool for True {}
// // pub enum False {}
// // impl Bool for False {}

// // trait Ternary {
// //     type Output;
// // }
// // impl<A, B> Ternary for (True, A, B) {
// //     type Output = A;
// // }
// // impl<A, B> Ternary for (False, A, B) {
// //     type Output = B;
// // }

// // trait IsEq {
// //     type Output: Bool = False;
// // }
// // impl<A> IsEq for (A, A) {
// //     type Output = True;
// // }

// // trait Id {
// //     type X;
// // }
// // impl<T> Id for T {
// //     type X = T;
// // }


// // type
