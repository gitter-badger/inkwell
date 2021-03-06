use llvm_sys::core::{LLVMGetBasicBlockParent, LLVMGetBasicBlockTerminator, LLVMGetNextBasicBlock, LLVMInsertBasicBlock, LLVMIsABasicBlock, LLVMIsConstant, LLVMMoveBasicBlockAfter, LLVMMoveBasicBlockBefore, LLVMPrintTypeToString, LLVMPrintValueToString, LLVMTypeOf, LLVMDeleteBasicBlock, LLVMGetPreviousBasicBlock, LLVMRemoveBasicBlockFromParent, LLVMGetFirstInstruction, LLVMGetLastInstruction};
use llvm_sys::prelude::{LLVMValueRef, LLVMBasicBlockRef};

use values::{FunctionValue, InstructionValue};

use std::fmt;
use std::ffi::{CStr, CString};

// Apparently BasicBlocks count as LabelTypeKinds, which is
// why they're allow to be casted to values?
#[derive(PartialEq, Eq)]
pub struct BasicBlock {
    pub(crate) basic_block: LLVMBasicBlockRef,
}

impl BasicBlock {
    pub(crate) fn new(basic_block: LLVMBasicBlockRef) -> Option<Self> {
        if basic_block.is_null() {
            return None;
        }

        unsafe {
            assert!(!LLVMIsABasicBlock(basic_block as LLVMValueRef).is_null()) // NOTE: There is a LLVMBasicBlockAsValue but it might be the same as casting
        }

        Some(BasicBlock { basic_block })
    }

    pub fn get_parent(&self) -> Option<FunctionValue> {
        let value = unsafe {
            LLVMGetBasicBlockParent(self.basic_block)
        };

        FunctionValue::new(value)
    }

    pub fn get_previous_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetPreviousBasicBlock(self.basic_block)
        };

        BasicBlock::new(bb)
    }

    pub fn get_next_basic_block(&self) -> Option<BasicBlock> {
        let bb = unsafe {
            LLVMGetNextBasicBlock(self.basic_block)
        };

        BasicBlock::new(bb)
    }

    // REVIEW: If we wanted the return type could be Option<Either<BasicValueEnum, InstructionValue>>
    // if getting a value over an instruction is preferable
    pub fn get_terminator(&self) -> Option<InstructionValue> {
        let value = unsafe {
            LLVMGetBasicBlockTerminator(self.basic_block)
        };

        if value.is_null() {
            return None;
        }

        Some(InstructionValue::new(value))
    }

    pub fn move_before(&self, basic_block: &BasicBlock) {
        unsafe {
            LLVMMoveBasicBlockBefore(self.basic_block, basic_block.basic_block)
        }
    }

    pub fn move_after(&self, basic_block: &BasicBlock) {
        unsafe {
            LLVMMoveBasicBlockAfter(self.basic_block, basic_block.basic_block)
        }
    }

    pub fn prepend_basic_block(&self, name: &str) -> BasicBlock {
        let c_string = CString::new(name).expect("Conversion to CString failed unexpectedly");

        let bb = unsafe {
            LLVMInsertBasicBlock(self.basic_block, c_string.as_ptr())
        };

        BasicBlock::new(bb).expect("Prepending basic block should never fail")
    }

    pub fn get_first_instruction(&self) -> Option<InstructionValue> {
        let value = unsafe {
            LLVMGetFirstInstruction(self.basic_block)
        };

        if value.is_null() {
            return None;
        }

        Some(InstructionValue::new(value))
    }

    pub fn get_last_instruction(&self) -> Option<InstructionValue> {
        let value = unsafe {
            LLVMGetLastInstruction(self.basic_block)
        };

        if value.is_null() {
            return None;
        }

        Some(InstructionValue::new(value))
    }

    // SubTypes: Don't need to call get_parent for a BasicBlock<HasParent>
    pub fn remove_from_function(&self) {
        // This method is UB if the parent no longer exists, so we must check for parent (or encode into type system)
        if self.get_parent().is_some() {
            unsafe {
                LLVMRemoveBasicBlockFromParent(self.basic_block)
            }
        }
    }

    // REVIEW: Could potentially be unsafe if there are existing references. Might need a global ref counter
    pub unsafe fn delete(self) {
        // unsafe {
        LLVMDeleteBasicBlock(self.basic_block)
        // }
    }
}

impl fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let llvm_value = unsafe {
            CStr::from_ptr(LLVMPrintValueToString(self.basic_block as LLVMValueRef))
        };
        let llvm_type = unsafe {
            CStr::from_ptr(LLVMPrintTypeToString(LLVMTypeOf(self.basic_block as LLVMValueRef)))
        };
        let is_const = unsafe {
            LLVMIsConstant(self.basic_block as LLVMValueRef) == 1
        };

        write!(f, "BasicBlock {{\n    address: {:?}\n    is_const: {:?}\n    llvm_value: {:?}\n    llvm_type: {:?}\n}}", self.basic_block, is_const, llvm_value, llvm_type)
    }
}
