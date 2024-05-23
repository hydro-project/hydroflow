//! Trait for the `demux_enum` derive and operator.

pub use hydroflow_macro::DemuxEnum;

/// Trait for use with the `demux_enum` operator.
///
/// This trait is meant to be derived: `#[derive(DemuxEnum)]`.
///
/// The derive will implement this such that `Outputs` can be any tuple where each item is a
/// `Pusherator` that corresponds to each of the variants of the tuple, in alphabetic order.
#[diagnostic::on_unimplemented(
    note = "Ensure there is exactly one output for each enum variant.",
    note = "Ensure that the type for each output is a tuple of the field for the variant: `()`, `(a,)`, or `(a, b, ...)`."
)]
pub trait DemuxEnum<Outputs>: DemuxEnumBase {
    /// Pushes self into the corresponding output pusherator in `outputs`.
    fn demux_enum(self, outputs: &mut Outputs);
}

/// Base implementation to constrain that `DemuxEnum<SOMETHING>` is implemented.
#[diagnostic::on_unimplemented(note = "Use `#[derive(hydroflow::DemuxEnum)]`")]
pub trait DemuxEnumBase {}
