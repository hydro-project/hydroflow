use super::OperatorConstraints;

/// An alias for [`keyed_fold`](#keyed_fold).
#[hydroflow_internalmacro::operator_docgen]
pub const GROUP_BY: OperatorConstraints = OperatorConstraints {
    name: "group_by",
    ..super::keyed_fold::KEYED_FOLD
};
