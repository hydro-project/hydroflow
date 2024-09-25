use syn::parse_quote_spanned;
use crate::graph::ops::{OperatorCategory, OperatorConstraints, PortListSpec, WriteContextArgs, RANGE_0, RANGE_1};

/// > Generic Argument: A enum type which has `#[derive(DemuxEnum)]`. Must match the items in the input stream.
///
/// Takes an input stream of enum instances and splits them into their variants.
///
/// ```rustdoc
/// #[derive(DemuxEnum)]
/// enum Shape {
///     Square(f64),
///     Rectangle(f64, f64),
///     Circle { r: f64 },
///     Triangle { w: f64, h: f64 }
/// }
///
/// let mut df = hydroflow_syntax! {
///     my_demux = source_iter([
///         Shape::Square(9.0),
///         Shape::Rectangle(10.0, 8.0),
///         Shape::Circle { r: 5.0 },
///         Shape::Triangle { w: 12.0, h: 13.0 },
///     ]) -> demux_enum::<Shape>();
///
///     my_demux[Square] -> map(|s| s * s) -> out;
///     my_demux[Circle] -> map(|(r,)| std::f64::consts::PI * r * r) -> out;
///     my_demux[Rectangle] -> map(|(w, h)| w * h) -> out;
///     my_demux[Circle] -> map(|(w, h)| 0.5 * w * h) -> out;
///
///     out = union() -> for_each(|area| println!("Area: {}", area));
/// };
/// df.run_available();
/// ```
pub const DEMUX_ENUM: OperatorConstraints = OperatorConstraints {
    name: "demux_enum",
    categories: &[OperatorCategory::MultiOut],
    hard_range_inn: RANGE_1,
    soft_range_inn: RANGE_1,
    hard_range_out: &(..),
    soft_range_out: &(..),
    num_args: 0,
    persistence_args: RANGE_0,
    type_args: RANGE_1,
    is_external_input: false,
    has_singleton_output: false,
    ports_inn: None,
    ports_out: Some(|| PortListSpec::Variadic),
    input_delaytype_fn: |_| None,
    write_fn: |wc @ &WriteContextArgs {
        op_span,
        ..
    },
               diagnostics| {
        let wc = WriteContextArgs {
            arguments: &parse_quote_spanned!(op_span => ::std::convert::identity),
            ..wc.clone()
        };

        (super::demux_enum_by::DEMUX_ENUM_BY.write_fn)(&wc, diagnostics)
    },
};