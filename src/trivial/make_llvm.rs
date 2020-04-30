use super::structure as i;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::transforms as llvmt;
use llvm_sys::*;
use std::collections::{HashMap, HashSet};

const UNNAMED: *const libc::c_char = b"\0".as_ptr() as *const libc::c_char;

struct Converter<'a> {
    source: &'a i::Program,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    main_fn: LLVMValueRef,

    value_pointers: HashMap<i::VariableId, LLVMValueRef>,
    input_pointer: LLVMValueRef,
    output_pointer: LLVMValueRef,
    label_blocks: Vec<LLVMBasicBlockRef>,
    current_block_terminated: bool,
}

impl<'a> Converter<'a> {
    fn u32_const(&self, value: u32) -> LLVMValueRef {
        unsafe { LLVMConstInt(LLVMInt32TypeInContext(self.context), value as u64, 0) }
    }

    fn store_value(&mut self, value: &i::Value, content: LLVMValueRef) {
        assert!(value.dimensions.len() == 0);
        match &value.base {
            i::ValueBase::Variable(id) => {
                let ptr = self
                    .value_pointers
                    .get(&id)
                    .expect("A variable was not given a pointer.");
                unsafe {
                    LLVMBuildStore(self.builder, content, *ptr);
                }
            }
            i::ValueBase::Literal(..) => panic!("Cannot store to a constant."),
        }
    }

    fn load_value(&mut self, value: &i::Value) -> LLVMValueRef {
        assert!(value.dimensions.len() == 0);
        match &value.base {
            i::ValueBase::Variable(id) => {
                let ptr = self
                    .value_pointers
                    .get(&id)
                    .expect("A variable was not given a pointer.");
                unsafe { LLVMBuildLoad(self.builder, *ptr, UNNAMED) }
            }
            i::ValueBase::Literal(lit) => match lit {
                i::KnownData::Array(..) => unimplemented!(),
                i::KnownData::Bool(value) => unsafe {
                    LLVMConstInt(
                        LLVMInt1TypeInContext(self.context),
                        if *value { 1 } else { 0 },
                        0,
                    )
                },
                i::KnownData::Int(value) => unsafe {
                    LLVMConstInt(LLVMInt32TypeInContext(self.context), *value as u64, 0)
                },
                i::KnownData::Float(value) => unsafe {
                    LLVMConstReal(LLVMFloatTypeInContext(self.context), *value as f64)
                },
            },
        }
    }

    fn get_block_for_label(&self, id: &i::LabelId) -> LLVMBasicBlockRef {
        self.label_blocks[id.raw()]
    }

    fn convert_binary_expression(
        &mut self,
        op: &i::BinaryOperator,
        a: &i::Value,
        b: &i::Value,
        x: &i::Value,
    ) {
        let result = match op {
            i::BinaryOperator::AddI => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildAdd(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::SubI => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildSub(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::MulI => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildMul(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::DivI => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildSDiv(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::ModI => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildSRem(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::AddF => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildFAdd(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::SubF => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildFSub(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::MulF => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildFMul(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::DivF => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildFDiv(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::ModF => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildFRem(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::CompI(condition) => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                let predicate = match condition {
                    i::Condition::Equal => LLVMIntPredicate::LLVMIntEQ,
                    i::Condition::NotEqual => LLVMIntPredicate::LLVMIntNE,
                    i::Condition::GreaterThan => LLVMIntPredicate::LLVMIntSGT,
                    i::Condition::GreaterThanOrEqual => LLVMIntPredicate::LLVMIntSGE,
                    i::Condition::LessThan => LLVMIntPredicate::LLVMIntSLT,
                    i::Condition::LessThanOrEqual => LLVMIntPredicate::LLVMIntSLE,
                };
                unsafe { LLVMBuildICmp(self.builder, predicate, ar, br, UNNAMED) }
            }
            i::BinaryOperator::CompF(condition) => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                let predicate = match condition {
                    i::Condition::Equal => LLVMRealPredicate::LLVMRealOEQ,
                    i::Condition::NotEqual => LLVMRealPredicate::LLVMRealONE,
                    i::Condition::GreaterThan => LLVMRealPredicate::LLVMRealOGT,
                    i::Condition::GreaterThanOrEqual => LLVMRealPredicate::LLVMRealOGE,
                    i::Condition::LessThan => LLVMRealPredicate::LLVMRealOLT,
                    i::Condition::LessThanOrEqual => LLVMRealPredicate::LLVMRealOLE,
                };
                unsafe { LLVMBuildFCmp(self.builder, predicate, ar, br, UNNAMED) }
            }
            i::BinaryOperator::BAnd => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildAnd(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::BOr => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildOr(self.builder, ar, br, UNNAMED) }
            }
            i::BinaryOperator::BXor => {
                let ar = self.load_value(a);
                let br = self.load_value(b);
                unsafe { LLVMBuildXor(self.builder, ar, br, UNNAMED) }
            }
        };
        self.store_value(x, result);
    }

    fn convert_move(&mut self, from: &i::Value, to: &i::Value) {
        let from = self.load_value(from);
        self.store_value(to, from);
    }

    fn convert_label(&mut self, id: &i::LabelId) {
        unsafe {
            if !self.current_block_terminated {
                LLVMBuildBr(self.builder, self.get_block_for_label(id));
            }
            LLVMPositionBuilderAtEnd(self.builder, self.get_block_for_label(id));
        }
        self.current_block_terminated = false;
    }

    fn convert_branch(
        &mut self,
        condition: &i::Value,
        true_target: &i::LabelId,
        false_target: &i::LabelId,
    ) {
        unsafe {
            LLVMBuildCondBr(
                self.builder,
                self.load_value(condition),
                self.get_block_for_label(true_target),
                self.get_block_for_label(false_target),
            );
        }
        self.current_block_terminated = true;
    }

    fn convert_jump(&mut self, label: &i::LabelId) {
        unsafe {
            LLVMBuildBr(self.builder, self.get_block_for_label(label));
        }
        self.current_block_terminated = true;
    }

    fn create_param_pointers(&mut self) {
        for (index, input) in self.source.borrow_inputs().iter().enumerate() {
            let mut indices = [self.u32_const(0), self.u32_const(index as u32)];
            let input_ptr = unsafe {
                LLVMBuildGEP(
                    self.builder,
                    self.input_pointer,
                    indices.as_mut_ptr(),
                    indices.len() as u32,
                    UNNAMED,
                )
            };
            self.value_pointers.insert(*input, input_ptr);
        }
        for (index, output) in self.source.borrow_outputs().iter().enumerate() {
            let mut indices = [self.u32_const(0), self.u32_const(index as u32)];
            let output_ptr = unsafe {
                LLVMBuildGEP(
                    self.builder,
                    self.output_pointer,
                    indices.as_mut_ptr(),
                    indices.len() as u32,
                    UNNAMED,
                )
            };
            self.value_pointers.insert(*output, output_ptr);
        }
    }

    fn create_variable_pointers(&mut self) {
        let (ins, outs) = (self.source.borrow_inputs(), self.source.borrow_outputs());
        let param_set: HashSet<_> = ins.iter().chain(outs.iter()).collect();
        for var_id in self.source.iterate_all_variables() {
            if param_set.contains(&var_id) {
                continue;
            }
            let llvmt = llvm_type(self.context, self.source[var_id].borrow_type());
            let var_ptr = unsafe { LLVMBuildAlloca(self.builder, llvmt, UNNAMED) };
            self.value_pointers.insert(var_id, var_ptr);
        }
    }

    fn create_blocks_for_labels(&mut self) {
        for label_index in 0..self.source.get_num_labels() {
            self.label_blocks.push(unsafe {
                LLVMAppendBasicBlockInContext(
                    self.context,
                    self.main_fn,
                    format!("l{}\0", label_index).as_ptr() as *const _,
                )
            });
        }
    }

    fn optimize(&mut self) {
        unsafe {
            let fpm = LLVMCreateFunctionPassManagerForModule(self.module);
            // Convert all our stores / loads into flat, efficient SSA style code.
            llvmt::scalar::LLVMAddScalarReplAggregatesPassSSA(fpm);
            llvmt::instcombine::LLVMAddInstructionCombiningPass(fpm);
            llvmt::scalar::LLVMAddReassociatePass(fpm);
            llvmt::scalar::LLVMAddGVNPass(fpm);
            llvmt::scalar::LLVMAddCFGSimplificationPass(fpm);
            LLVMInitializeFunctionPassManager(fpm);

            assert!(
                LLVMRunFunctionPassManager(fpm, self.main_fn) == 1,
                "Optimization failed!"
            );
        }
    }

    fn convert(mut self) {
        self.create_param_pointers();
        self.create_variable_pointers();
        self.create_blocks_for_labels();

        for instruction in self.source.borrow_instructions().clone() {
            match instruction {
                i::Instruction::BinaryOperation { op, a, b, x } => {
                    self.convert_binary_expression(op, a, b, x)
                }
                i::Instruction::Move { from, to } => self.convert_move(from, to),
                i::Instruction::Label(id) => self.convert_label(id),
                i::Instruction::Branch {
                    condition,
                    true_target,
                    false_target,
                } => self.convert_branch(condition, true_target, false_target),
                i::Instruction::Jump { label } => self.convert_jump(label),
                _ => unimplemented!("{:?}", instruction),
            }
        }

        unsafe {
            LLVMBuildRetVoid(self.builder);
            LLVMDisposeBuilder(self.builder);

            // Dump human-readable IR to stdout
            println!("\nUnoptimized:");
            LLVMDumpModule(self.module);
        }

        self.optimize();

        unsafe {
            println!("\nOptimized:");
            LLVMDumpModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}

fn llvm_type(context: LLVMContextRef, trivial_type: &i::DataType) -> LLVMTypeRef {
    let mut llvm_type = unsafe {
        match trivial_type.get_base() {
            i::BaseType::B1 => LLVMInt1TypeInContext(context),
            i::BaseType::I32 => LLVMInt32TypeInContext(context),
            i::BaseType::F32 => LLVMFloatTypeInContext(context),
        }
    };
    for dim in trivial_type.borrow_dimensions().iter().rev() {
        llvm_type = unsafe { LLVMArrayType(llvm_type, *dim as u32) };
    }
    llvm_type
}

pub fn make_llvm(source: &i::Program) {
    unsafe {
        let context = LLVMContextCreate();
        let module = LLVMModuleCreateWithNameInContext(b"nsprog\0".as_ptr() as *const _, context);
        let builder = LLVMCreateBuilderInContext(context);

        let mut input_types = Vec::new();
        for input in source.borrow_inputs() {
            let var = source.borrow_variable(*input);
            input_types.push(llvm_type(context, var.borrow_type()));
        }
        let input_data_type = LLVMStructType(input_types.as_mut_ptr(), input_types.len() as u32, 0);
        let input_pointer_type = LLVMPointerType(input_data_type, 0);

        let mut output_types = Vec::new();
        for output in source.borrow_outputs() {
            let var = source.borrow_variable(*output);
            output_types.push(llvm_type(context, var.borrow_type()));
        }
        let output_data_type =
            LLVMStructType(output_types.as_mut_ptr(), output_types.len() as u32, 0);
        let output_pointer_type = LLVMPointerType(output_data_type, 0);

        let voidt = LLVMVoidTypeInContext(context);
        let mut argts = [input_pointer_type, output_pointer_type];
        let function_type = LLVMFunctionType(voidt, argts.as_mut_ptr(), argts.len() as u32, 0);
        let main_fn = LLVMAddFunction(module, b"main\0".as_ptr() as *const _, function_type);
        let entry_block =
            LLVMAppendBasicBlockInContext(context, main_fn, b"entry\0".as_ptr() as *const _);
        LLVMPositionBuilderAtEnd(builder, entry_block);
        let input_pointer = LLVMGetParam(main_fn, 0);
        let output_pointer = LLVMGetParam(main_fn, 1);

        let converter = Converter {
            source,
            context,
            module,
            builder,
            main_fn,

            value_pointers: HashMap::new(),
            input_pointer,
            output_pointer,
            label_blocks: Vec::with_capacity(source.get_num_labels()),
            current_block_terminated: false,
        };

        converter.convert()
    }
}