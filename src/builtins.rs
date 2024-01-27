use indexmap::IndexMap;
use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum, FunctionType},
    AddressSpace,
};

pub fn get_val_type<'ctx>(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
    context
        .struct_type(&[context.i8_type().into()], true)
        .ptr_type(AddressSpace::default())
        .into()
}

pub fn create_builtin_functions<'ctx>(
    context: &'ctx Context,
) -> IndexMap<&'static str, FunctionType<'ctx>> {
    let mut map = IndexMap::new();

    let val_type = get_val_type(context);

    map.insert("new_null_val", val_type.fn_type(&[], false));

    map.insert(
        "new_bool_val",
        val_type.fn_type(&[context.bool_type().into()], false),
    );

    map.insert(
        "new_int_val",
        val_type.fn_type(&[context.i64_type().into()], false),
    );

    map.insert(
        "new_float_val",
        val_type.fn_type(&[context.f64_type().into()], false),
    );

    map.insert(
        "new_str_val",
        val_type.fn_type(
            &[context.i8_type().ptr_type(AddressSpace::default()).into()],
            false,
        ),
    );

    map.insert("val_get_type", val_type.fn_type(&[val_type.into()], false));
    map.insert(
        "val_get_value",
        val_type.fn_type(
            &[
                val_type.into(),
                context.i8_type().ptr_type(AddressSpace::default()).into(),
            ],
            false,
        ),
    );

    map.insert(
        "new_array_val",
        val_type.fn_type(&[context.i64_type().into()], false),
    );

    map.insert("new_object_val", val_type.fn_type(&[], false));

    map.insert(
        "val_op_add",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_sub",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_mul",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_div",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_mod",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert("val_op_pos", val_type.fn_type(&[val_type.into()], false));
    map.insert("val_op_neg", val_type.fn_type(&[val_type.into()], false));
    map.insert("val_op_not", val_type.fn_type(&[val_type.into()], false));

    map.insert(
        "val_op_eq",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_neq",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_seq",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_sneq",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_gt",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_gte",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_lt",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_op_lte",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_array_push",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map.insert(
        "val_object_set",
        val_type.fn_type(
            &[
                val_type.into(),
                context.i8_type().ptr_type(AddressSpace::default()).into(),
                val_type.into(),
            ],
            false,
        ),
    );

    map.insert(
        "val_object_get",
        val_type.fn_type(
            &[
                val_type.into(),
                context.i8_type().ptr_type(AddressSpace::default()).into(),
            ],
            false,
        ),
    );

    map.insert("link_val", val_type.fn_type(&[val_type.into()], false));
    map.insert("unlink_val", val_type.fn_type(&[val_type.into()], false));

    map
}
