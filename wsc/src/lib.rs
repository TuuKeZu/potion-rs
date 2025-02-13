use std::{error::Error, io};

use swc::{config::{IsModule, SourceMapsConfig}, Compiler, PrintArgs};
use swc_common::{comments::SingleThreadedComments, errors::Handler, source_map::{DefaultSourceMapGenConfig, SourceMap}, sync::Lrc, BytePos, Globals, Mark, GLOBALS};
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::{text_writer::WriteJs, to_code, to_code_default, Emitter, Node};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_transforms_base::{fixer, hygiene::hygiene, resolver};
use swc_ecma_transforms_typescript::strip;
use swc_ecma_transforms_base::fixer::fixer;
use anyhow::Context;


pub fn ts_to_js(filename: &str, ts_code: &str) -> Result<(String, String), Box<dyn Error>> {
    let cm: Lrc<SourceMap> = Default::default();
    let args = PrintArgs::default();
    
    let source = cm.new_source_file(
        swc_common::FileName::Custom(filename.into()).into(),
        ts_code.to_string(),
    );

    let comments = SingleThreadedComments::default();
    
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: false,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*source),
        Some(&comments),
    );

    let mut parser = Parser::new_from(lexer);
    let handler = Handler::with_emitter_writer(Box::new(io::stderr()), Some(cm.clone()));

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    
    let module = parser
    .parse_program()
    .map_err(|e| e.into_diagnostic(&handler).emit())
    .expect("failed to parse module.");


    let globals = Globals::default();

    return GLOBALS.set(&globals, || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        let module = module.apply(resolver(unresolved_mark, top_level_mark, true));
        let module = module.apply(strip(unresolved_mark, top_level_mark));
        let module = module.apply(hygiene());

        let program = module.apply(fixer(None));

        // let a = to_code_default(cm.clone(), Some(&comments), &program);

        let mut src_map_buf = std::vec::Vec::new();
        let code = {
            let mut buf = std::vec::Vec::new();
            {
                let mut w = swc_ecma_codegen::text_writer::JsWriter::new(
                    cm.clone(),
                    "\n",
                    &mut buf,
                    Some(&mut src_map_buf)
                );
        
                w.preamble(PrintArgs::default().preamble).unwrap();
                let mut wr = Box::new(w) as Box<dyn WriteJs>;
                wr = Box::new(swc_ecma_codegen::text_writer::omit_trailing_semi(wr));
        
                let mut emitter = Emitter {
                    cfg: args.codegen_config,
                    comments: Some(&comments),
                    cm: cm.clone(),
                    wr,
                };
        
                (&program).emit_with(&mut emitter)?;
            }
            
    
            String::from_utf8(buf)?
        };

        let map = cm.build_source_map_with_config(
                &src_map_buf,
                args.orig,
                DefaultSourceMapGenConfig,
            );
        
        let mut _buf = std::vec::Vec::new();
        map
            .to_writer(&mut _buf)
            .context("failed to write source map")?;
        
        let map = String::from_utf8(_buf).context("source map is not utf-8")?;

        return Ok((code, map));
    });
}
