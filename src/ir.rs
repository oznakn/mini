use cranelift_codegen::ir::types;
use cranelift_codegen::ir::AbiParam;
use cranelift_codegen::ir::Signature;
use cranelift_codegen::isa::CallConv;

use crate::value::*;

impl VariableKind {
    pub fn get_abi(&self) -> Option<AbiParam> {
        match self {
            VariableKind::Number => Some(AbiParam::new(types::I64)),
            _ => None,
        }
    }
}

impl FunctionKind {
    pub fn get_signature(&self) -> Signature {
        let mut signature = Signature::new(CallConv::SystemV);
        for parameter in self.parameters.iter() {
            if let Some(param) = parameter.get_abi() {
                signature.params.push(param);
            }
        }

        if let Some(param) = self.return_kind.get_abi() {
            signature.returns.push(param);
        }

        signature
    }
}
