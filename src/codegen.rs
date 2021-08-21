use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple};
use std::path::Path;
use anyhow::Result;

pub type OptLevel = OptimizationLevel;

pub struct CodegenOptions<'a> {
    output: &'a Path,
    optimization: OptLevel,
    /// Position Independent Code
    pic: bool,
    /// Target triple, None for host
    target: Option<String>,
}

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn compile(/* ast: Ast, */options: CodegenOptions) -> Result<()> {
        let context = Context::create();
        let module = context.create_module("beat saber");
        let mut codegen = Codegen {
            context: &context,
            module,
            builder: context.create_builder(),
        };

        // TODO: Compile ast

        codegen.write_object(options)
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
            .create_target_machine(
                &triple,
                &cpu,
                &features,
                opt,
                reloc,
                model,
            ).unwrap();

        target_machine
            .write_to_file(&self.module, FileType::Object, options.output).unwrap();

        Ok(())
    }
}
