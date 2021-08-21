use crate::ast2::{self, FuncBlock};
use anyhow::Result;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use inkwell::types::{BasicTypeEnum, IntType};
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::Path;

pub type OptLevel = OptimizationLevel;

pub struct CodegenOptions<'a> {
    pub output: &'a Path,
    pub optimization: OptLevel,
    /// Position Independent Code
    pub pic: bool,
    /// Target triple, None for host
    pub target: Option<String>,
}

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    i64: IntType<'ctx>,
    func_compile_queue: Vec<FuncBlock>,
    functions: HashMap<usize, FunctionValue<'ctx>>,

    cur_locals: HashMap<usize, PointerValue<'ctx>>,
    cur_func: Option<FunctionValue<'ctx>>,
    cur_line_map: HashMap<usize, BasicBlock<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn compile(ast: Vec<ast2::DecoratedStmt>, options: CodegenOptions) -> Result<()> {
        let context = Context::create();
        let module = context.create_module("beat saber");

        let mut codegen = Codegen {
            context: &context,
            module,
            builder: context.create_builder(),
            i64: context.i64_type(),
            func_compile_queue: Vec::new(),
            functions: HashMap::new(),

            cur_locals: HashMap::new(),
            cur_func: None,
            cur_line_map: HashMap::new(),
        };

        codegen.declare_func_children(&ast);
        codegen.build_main(ast);

        codegen.write_object(options)
    }

    fn declare_func_children(&mut self, stmts: &[ast2::DecoratedStmt]) {
        for stmt in stmts {
            match stmt {
                ast2::DecoratedStmt::Callable(ast2::Callable::ExternFunction(stmt)) => {
                    let mut param_types = vec![
                        BasicTypeEnum::IntType(self.i64),
                        BasicTypeEnum::IntType(self.i64),
                    ];
                    let fn_type = self.i64.fn_type(&param_types, false);
                    let fn_val =
                        self.module
                            .add_function(&stmt.name, fn_type, Some(Linkage::External));
                    self.functions.insert(stmt.ident.id, fn_val);
                }
                ast2::DecoratedStmt::Callable(ast2::Callable::FuncBlock(stmt)) => {
                    let mut param_types = vec![BasicTypeEnum::IntType(self.i64)];
                    if stmt.decl.p2.is_some() {
                        param_types.push(BasicTypeEnum::IntType(self.i64));
                    }

                    let fn_type = self.i64.fn_type(&param_types, false);
                    let fn_val = self.module.add_function(
                        &stmt.decl.id.id.to_string(),
                        fn_type,
                        Some(Linkage::Internal),
                    );
                    self.functions.insert(stmt.decl.id.id, fn_val);

                    self.declare_func_children(&stmt.block);
                }
                _ => {}
            }
        }
    }

    fn get_local(&mut self, id: usize) -> PointerValue<'ctx> {
        if let Some(local) = self.cur_locals.get(&id) {
            return *local;
        }

        let builder = self.context.create_builder();

        let entry = self.cur_func.unwrap().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        let ptr = builder.build_alloca(self.i64, "");
        self.cur_locals.insert(id, ptr);
        ptr
    }

    fn build_main(&mut self, body: Vec<ast2::DecoratedStmt>) {
        let fn_type = self.context.i32_type().fn_type(&[], false);
        let fn_val = self
            .module
            .add_function("main", fn_type, Some(Linkage::External));
        self.cur_func = Some(fn_val);

        for stmt in &body {
            let line = stmt.line_number();
            if matches!(stmt, ast2::DecoratedStmt::Callable(_)) {
                continue;
            }
            let block = self.context.append_basic_block(fn_val, "");
            self.cur_line_map.insert(line, block);
        }

        for stmt in body {
            self.build_stmt(stmt);
        }

        while !self.func_compile_queue.is_empty() {
            let func = self.func_compile_queue.pop().unwrap();
            let p1 = func.decl.p1.id;
            let p2 = func.decl.p2.map(|id| id.id);
            self.build_func(func.block, func.decl.id.id, p1, p2);
        }
    }

    fn build_func(&mut self, body: Vec<ast2::DecoratedStmt>, id: usize, p1: usize, p2: Option<usize>) {
        self.cur_locals.clear();
        self.cur_line_map.clear();
        let fn_val = *self.functions.get(&id).unwrap();
        self.cur_func = Some(fn_val);

        let params = fn_val.get_params();

        let entry = self.context.append_basic_block(fn_val, "");
        self.builder.position_at_end(entry);
        let p1alloca = self.builder.build_alloca(self.i64, "");
        self.builder.build_store(p1alloca, params[0]);
        self.cur_locals.insert(p1, p1alloca);

        if let Some(p2) = p2 {
            let p2alloca = self.builder.build_alloca(self.i64, "");
            self.builder.build_store(p2alloca, params[1]);
            self.cur_locals.insert(p2, p2alloca);
        }

        for stmt in &body {
            let line = stmt.line_number();
            if matches!(stmt, ast2::DecoratedStmt::Callable(_)) {
                continue;
            }
            let block = self.context.append_basic_block(fn_val, "");
            self.cur_line_map.insert(line, block);
        }

        self.builder.build_unconditional_branch(entry.get_next_basic_block().unwrap());

        for stmt in body {
            self.build_stmt(stmt);
        }

        // if let Some(func) = self.cur_func {
        //     if func.verify(true) {
        //     } else {
        //         unsafe {
        //             func.delete();
        //         }
    
        //         panic!("stack bad");
        //     }
        // }
    }

    fn build_expr(&mut self, expr: ast2::DecoratedExpr) -> IntValue<'ctx> {
        match expr {
            ast2::DecoratedExpr::CallExpr(expr) => {
                let fn_val = self.functions[&expr.function.id];
                let mut args = Vec::new();
                let p1 = self.build_expr(*expr.p1);
                args.push(BasicValueEnum::IntValue(p1));
                if let Some(p2) = expr.p2 {
                    args.push(BasicValueEnum::IntValue(self.build_expr(*p2)));
                }
                self.builder
                    .build_call(fn_val, &args, "")
                    .try_as_basic_value()
                    .left()
                    .unwrap()
                    .into_int_value()
            }
            ast2::DecoratedExpr::Identifier(expr) => {
                let ptr = self.get_local(expr.id);
                self.builder.build_load(ptr, "").into_int_value()
            }
        }
    }

    fn build_stmt(&mut self, stmt: ast2::DecoratedStmt) {
        match stmt {
            ast2::DecoratedStmt::Callable(stmt) => match stmt {
                ast2::Callable::FuncBlock(stmt) => {
                    // println!("Pushing function block to compile queue {:?}", stmt);
                    self.func_compile_queue.push(stmt);
                    return;
                }
                _ => return,
            }
            _ => {}
        }

        let line = stmt.line_number();
        let block = self.cur_line_map[&line];
        self.builder.position_at_end(block);
        match stmt {
            ast2::DecoratedStmt::LoadLiteralNumber(stmt) => {
                let ptr = self.get_local(stmt.ident.id);
                let val = self.i64.const_int(stmt.value as u64, false);
                self.builder.build_store(ptr, val);
            }
            ast2::DecoratedStmt::Conditional(stmt) => {
                let cond = self.build_expr(ast2::DecoratedExpr::Identifier(stmt.condition));
                let then_block = self.context.append_basic_block(self.cur_func.unwrap(), "");
                // shitty hack to get it to write this statement to the then block
                self.cur_line_map.insert(line, then_block);
                self.build_stmt(*stmt.success);
                self.cur_line_map.insert(line, block);

                let else_block = block.get_next_basic_block().unwrap();
                self.builder.position_at_end(block);
                self.builder
                    .build_conditional_branch(cond, then_block, else_block);
            }
            ast2::DecoratedStmt::Assignment(stmt) => {
                if let Some(id) = stmt.name {
                    let ptr = self.get_local(id.id);
                    let val = self.build_expr(stmt.value);
                    self.builder.build_store(ptr, val);
                }
            }
            ast2::DecoratedStmt::ReturnStmt(stmt) => {
                let val = self.build_expr(stmt.expr);
                self.builder.build_return(Some(&val));
            }
            ast2::DecoratedStmt::GotoStmt(stmt) => {}
            _ => unreachable!()
        }
        if let Some(next_block) = block.get_next_basic_block() {
            self.builder.build_unconditional_branch(next_block);
        }
    }

    fn write_object(&self, options: CodegenOptions) -> Result<()> {
        Target::initialize_all(&InitializationConfig::default());

        let triple = if let Some(triple) = &options.target {
            TargetTriple::create(triple)
        } else {
            TargetMachine::get_default_triple()
        };
        let target = Target::from_triple(&triple).unwrap();
        let (cpu, features) = if options.target.is_some() {
            // TODO: cli option for cpu and features
            (String::new(), String::new())
        } else {
            let cpu = TargetMachine::get_host_cpu_name().to_string();
            let features = TargetMachine::get_host_cpu_features().to_string();
            (cpu, features)
        };

        let reloc = if options.pic {
            RelocMode::PIC
        } else {
            RelocMode::Default
        };
        let model = CodeModel::Default;
        let opt = options.optimization;

        let target_machine = target
            .create_target_machine(&triple, &cpu, &features, opt, reloc, model)
            .unwrap();

        self.module.print_to_stderr();
        target_machine
            .write_to_file(&self.module, FileType::Object, options.output)
            .unwrap();

        Ok(())
    }
}
