use super::OperatorConstraints;

/// An alias for [`fold_keyed`](#fold_keyed).
#[hydroflow_internalmacro::operator_docgen]
pub const GROUP_BY: OperatorConstraints = OperatorConstraints {
    name: "group_by",
    ..super::fold_keyed::FOLD_KEYED
};
