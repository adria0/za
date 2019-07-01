
use crate::{
    evaluator::{Evaluator},
    storage::{Signals,Constraints},
};

use codespan::{ByteSpan, CodeMap, Span};
use codespan_reporting::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::{emit, Diagnostic, Label, Severity};

pub fn dump_error<S: Signals, C: Constraints>(eval: &Evaluator<S, C>, err: &str) {
    let msg = format!("{}", err);

    if let Some(ctx) = &eval.last_error {
        let span: ByteSpan = Span::from_offset(
            (1 + ctx.meta.start as u32).into(),
            (1 + (ctx.meta.end - ctx.meta.start) as i64).into(),
        );

        println!("SCOPE DUMP ------------------------------------------------");
        println!("{}", ctx.scope);

        if ctx.file != "" {
            println!("Located in {}:{}",ctx.file,ctx.meta.start );

            let mut code_map = CodeMap::new();
            code_map
                .add_filemap_from_disk(&ctx.file)
                .unwrap_or_else(|_| panic!("cannot read source file '{}'", &ctx.file));

            let error = Diagnostic::new(Severity::Error, "Failed to execute")
                .with_label(Label::new_primary(span).with_message(msg.clone()));

            let writer = StandardStream::stderr(ColorChoice::Always);
            emit(&mut writer.lock(), &code_map, &error).unwrap();
        } else {
            println!("No ctx.file located {}",ctx.file);
        }
    } 
}
