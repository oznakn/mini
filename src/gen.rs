use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::*;
use cranelift_codegen::isa;
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings;
use cranelift_codegen::Context;
use cranelift_frontend::*;
use cranelift_module::*;
use cranelift_object::*;

use crate::error::CompilerError;

pub struct IRGenerator {
    pub module: ObjectModule,
}

impl<'input> IRGenerator {
    pub fn new(arch: &str, name: &str) -> Result<Self, CompilerError<'input>> {
        let flag_builder = settings::builder();

        let isa_builder = isa::lookup_by_name(arch)
            .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|err| CompilerError::CodeGenError(err.to_string()))?;

        let module = ObjectModule::new(
            ObjectBuilder::new(isa, name, default_libcall_names())
                .map_err(|err| CompilerError::CodeGenError(err.to_string()))?,
        );

        Ok(IRGenerator { module })
    }

    #[allow(dead_code)]
    fn init(&mut self) -> Result<(), CompilerError<'input>> {
        Ok(())
    }

    pub fn add_function_with_signature(
        &mut self,
        mut ctx: Context,
    ) -> Result<Context, CompilerError<'input>> {
        let mut func_ctx = FunctionBuilderContext::new();
        {
            let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let start = bcx.create_block();

            {
                bcx.switch_to_block(start);
                bcx.seal_block(start);

                let v = Variable::new(0);
                bcx.declare_var(v, types::I32);

                let tmp = bcx.ins().iconst(types::I32, 2);
                bcx.def_var(v, tmp);

                let r = bcx.use_var(v);
                bcx.ins().return_(&[r]);
            }

            bcx.finalize();
        }

        Ok(ctx)
    }

    pub fn add_function(&mut self) -> Result<(), CompilerError<'input>> {
        let mut function_signature = Signature::new(CallConv::SystemV);
        function_signature.returns.push(AbiParam::new(types::I32));

        let id = self
            .module
            .declare_function("main", Linkage::Export, &function_signature)
            .unwrap();

        let mut ctx = Context::new();
        ctx.func = Function::with_name_signature(UserFuncName::user(0, 0), function_signature);

        ctx = self.add_function_with_signature(ctx).unwrap();

        self.module.define_function(id, &mut ctx).unwrap();

        println!("{}", ctx.func.display());

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), CompilerError<'input>> {
        self.add_function().unwrap();

        Ok(())
    }
}
