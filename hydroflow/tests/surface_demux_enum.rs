use hydroflow::util::demux_enum::DemuxEnum;
use hydroflow::{hydroflow_syntax, var_args, var_expr, var_type};
use multiplatform_test::multiplatform_test;
use pusherator::for_each::ForEach;

// #[test]
// fn test() {
//     use pusherator::demux::PusheratorList;
//     use pusherator::Pusherator;

//     enum MyEnum {
//         Square(usize),
//         Rectangle { w: usize, h: usize },
//         Circle(usize),
//     }
//     impl<Square, Rectangle, Circle> DemuxEnum<var_type!(Square, Rectangle, Circle)> for MyEnum
//     where
//         Square: Pusherator<Item = usize>,
//         Rectangle: Pusherator<Item = (usize, usize)>,
//         Circle: Pusherator<Item = usize>,
//         var_type!(Square, Rectangle, Circle): PusheratorList,
//     {
//         type This = Self;

//         fn demux_enum(self, var_args!(sq, re, ci): &mut var_type!(Square, Rectangle, Circle)) {
//             match self {
//                 MyEnum::Square(s) => sq.give(s),
//                 MyEnum::Rectangle { w, h } => re.give((w, h)),
//                 MyEnum::Circle(r) => ci.give(r),
//             }
//         }
//     }

//     let val = MyEnum::Rectangle { w: 5, h: 6 };
//     let mut nexts = var_expr!(
//         ForEach::new(|x| println!("1 {:?}", x)),
//         ForEach::new(|x| println!("2 {:?}", x)),
//         ForEach::new(|x| println!("3 {:?}", x)),
//     );
//     val.demux_enum(&mut nexts);
// }

// #[test]
// fn test2() {
//     #[derive(DemuxEnum)]
//     enum MyEnum2 {
//         Square(usize),
//         Rectangle { w: usize, h: usize },
//         Circle { r: usize },
//     }

//     let val = MyEnum2::Rectangle { w: 5, h: 6 };
//     let mut nexts = var_expr!(
//         ForEach::new(|x| println!("1 {:?}", x)),
//         ForEach::new(|x| println!("2 {:?}", x)),
//         ForEach::new(|x| println!("3 {:?}", x)),
//     );
//     val.demux_enum(&mut nexts);
// }

#[multiplatform_test]
pub fn test_demux_enum() {
    #[derive(DemuxEnum)]
    enum Shape {
        Square(f64),
        Rectangle { w: f64, h: f64 },
        Circle { r: f64 },
    }

    let mut df = hydroflow_syntax! {
        my_demux = source_iter([
            Shape::Rectangle { w: 10.0, h: 8.0 },
            Shape::Square(9.0),
            Shape::Circle { r: 5.0 },
        ]) -> demux_enum::<Shape>();

        my_demux[Circle] -> map(|(r,)| std::f64::consts::PI * r * r) -> out;
        my_demux[Rectangle] -> map(|(w, h)| w * h) -> out;
        my_demux[Square] -> null(); //map(|s| s * s) -> out;

        out = union() -> for_each(|area| println!("Area: {}", area));
    };
    df.run_available();
}
