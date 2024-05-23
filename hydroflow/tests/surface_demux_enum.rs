use hydroflow::hydroflow_syntax;
use hydroflow::util::demux_enum::{DemuxEnum, DemuxEnumBase};
use multiplatform_test::multiplatform_test;
use pusherator::for_each::ForEach;

#[multiplatform_test]
fn test_manual_impl() {
    use pusherator::Pusherator;

    enum Shape {
        Square(usize),
        Rectangle { w: usize, h: usize },
        Circle { r: usize },
    }
    impl<Square, Rectangle, Circle> DemuxEnum<(Square, Rectangle, Circle)> for Shape
    where
        Square: Pusherator<Item = usize>,
        Rectangle: Pusherator<Item = (usize, usize)>,
        Circle: Pusherator<Item = (usize,)>,
    {
        fn demux_enum(self, (sq, re, ci): &mut (Square, Rectangle, Circle)) {
            match self {
                Self::Square(s) => sq.give(s),
                Self::Rectangle { w, h } => re.give((w, h)),
                Self::Circle { r } => ci.give((r,)),
            }
        }
    }
    impl DemuxEnumBase for Shape {}

    let vals = [
        Shape::Square(5),
        Shape::Rectangle { w: 5, h: 6 },
        Shape::Circle { r: 6 },
    ];
    let mut nexts = (
        ForEach::new(|x| println!("1 {:?}", x)),
        ForEach::new(|x| println!("2 {:?}", x)),
        ForEach::new(|x| println!("3 {:?}", x)),
    );
    for val in vals {
        val.demux_enum(&mut nexts);
    }
}

#[multiplatform_test]
fn test_derive() {
    #[derive(DemuxEnum)]
    enum Shape {
        Square(usize),
        Rectangle { w: usize, h: usize },
        Circle { r: usize },
    }

    let vals = [
        Shape::Square(5),
        Shape::Rectangle { w: 5, h: 6 },
        Shape::Circle { r: 6 },
    ];
    let mut nexts = (
        ForEach::new(|x| println!("1 {:?}", x)),
        ForEach::new(|x| println!("2 {:?}", x)),
        ForEach::new(|x| println!("3 {:?}", x)),
    );
    for val in vals {
        val.demux_enum(&mut nexts);
    }
}

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
            Shape::Square(9.0),
            Shape::Rectangle { w: 10.0, h: 8.0 },
            Shape::Circle { r: 5.0 },
        ]) -> demux_enum::<Shape>();

        my_demux[Square] -> map(|(s,)| s * s) -> out;
        my_demux[Circle] -> map(|(r,)| std::f64::consts::PI * r * r) -> out;
        my_demux[Rectangle] -> map(|(w, h)| w * h) -> out;

        out = union() -> for_each(|area| println!("Area: {}", area));
    };
    df.run_available();
}
