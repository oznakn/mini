use indexmap::IndexMap;
use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum, FunctionType},
    values::BasicValueEnum,
    AddressSpace,
};

pub fn get_null_value<'ctx>(context: &'ctx Context) -> BasicValueEnum<'ctx> {
    get_val_type(context).const_zero().into()
}

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

    map.insert(
        "new_str",
        val_type.fn_type(
            &[context.i8_type().ptr_type(AddressSpace::default()).into()],
            false,
        ),
    );

    map.insert(
        "new_int",
        val_type.fn_type(&[context.i64_type().into()], false),
    );

    map.insert(
        "val_op_plus",
        val_type.fn_type(&[val_type.into(), val_type.into()], false),
    );

    map
}
