use indexmap::IndexMap;
use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum, FunctionType},
    values::BasicValueEnum,
    AddressSpace,
};

pub fn get_null_value<'ctx>(context: &'ctx Context) -> BasicValueEnum<'ctx> {
    context.i64_type().const_zero().into()
}

pub fn get_string_type<'ctx>(context: &'ctx Context) -> BasicTypeEnum<'ctx> {
    context
        .struct_type(&[context.i8_type().into()], true)
        .ptr_type(AddressSpace::default())
        .into()
}

pub fn create_builtin_functions<'ctx>(
    context: &'ctx Context,
) -> IndexMap<&'static str, FunctionType<'ctx>> {
    let mut map = IndexMap::new();

    let string_type = get_string_type(context);

    map.insert(
        "new_str",
        string_type.fn_type(
            &[context.i8_type().ptr_type(AddressSpace::default()).into()],
            false,
        ),
    );

    map.insert(
        "str_length",
        context.i64_type().fn_type(&[string_type.into()], false),
    );

    map.insert(
        "str_combine",
        string_type.fn_type(&[string_type.into(), string_type.into()], false),
    );

    map
}
