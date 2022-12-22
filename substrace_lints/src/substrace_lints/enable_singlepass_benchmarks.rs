use super::auxiliary::paths;
use substrace_utils::diagnostics::span_lint_and_sugg;
use substrace_utils::source::{snippet_opt, line_span};
use substrace_utils::match_def_path;
use substrace_utils::is_in_cfg_test;
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_ast as ast;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::source_map::{SourceMap};

use serde::{Deserialize, Serialize};

use std::ffi::OsString;
use std::error::Error;

use termcolor::ColorChoice;
use walkdir::WalkDir;

use grep::cli;
use grep::cli::{StandardStream};
use grep::printer::{ColorSpecs, StandardBuilder};
use grep::regex::RegexMatcher;
use grep::searcher::{BinaryDetection, SearcherBuilder};

use super::extrinsics_must_be_tagged::is_extrinsic_name;

declare_lint! {
    pub ENABLE_SINGLEPASS_BENCHMARKS,
    Warn,
    "TODO"
}

impl_lint_pass!(EnableSinglepassBenchmarks => [ENABLE_SINGLEPASS_BENCHMARKS]);

#[derive(Clone, Default)]
pub struct EnableSinglepassBenchmarks;

#[derive(Debug, Deserialize, Serialize)]
struct MyGrepResult {
    #[serde(rename = "type")] 
    type_name: String,
    data: serde_json::Value,
}

impl<'tcx> LateLintPass<'tcx> for EnableSinglepassBenchmarks {

    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        println!("Checkin crate!");


        let my_str = std::fs::read_to_string("output.jsonl").ok().unwrap();
        // println!("mystring: {:?}", my_str);

        let hir_id = cx.last_node_with_lint_attrs;

        println!("hir: {:?}", hir_id);
        

        if let hir::Node::Crate(hir::Mod{spans, ..}) = cx.tcx.hir().find(hir_id).unwrap() {
            for line in my_str.lines() {
            
                let the_json: MyGrepResult = serde_json::from_str(&line).unwrap();
    
                if the_json.type_name == "match" {
                    println!("Matched file: {:?} at {:?}", the_json.data["path"]["text"], the_json.data["line_number"]);
    
    
                    span_lint_and_sugg(
                        cx,
                        ENABLE_SINGLEPASS_BENCHMARKS,
                        spans.inner_span,
                        "substrace: blabla test runtimes benchmarks",
                        "test jooo",
                        format!("En dan hier suggestion"),
                        Applicability::MachineApplicable, // Suggestion can be applied automatically
                    );
                    
                }
    
                // println!("Json: {:?}, value: {:?}", the_json, the_json.type_name);
            }
        }

        

    }
}





fn try_main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<OsString> = vec!{};
    // if args.len() < 2 {
    //     return Err("Usage: simplegrep <pattern> [<path> ...]".into());
    // }
    // if args.len() == 2 {
        args.push(OsString::from("nowwtf"));
        args.push(OsString::from("t"));
        args.push(OsString::from("./src/"));
    // }
    search(cli::pattern_from_os(&args[1])?, &args[2..])
}

fn search(pattern: &str, paths: &[OsString]) -> Result<(), Box<dyn Error>> {
    println!("paths {:?}", paths);
    let matcher = RegexMatcher::new_line_matcher(&pattern)?;
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(false)
        .build();
    let mut printer = StandardBuilder::new()
        .color_specs(ColorSpecs::default_with_color())
        .build(cli::stdout(if cli::is_tty_stdout() {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        }));

    for path in paths {
        for result in WalkDir::new(path) {
            // println!("WHAT RESULT: {:?}", result);
            let dent = match result {
                Ok(dent) => dent,
                Err(err) => {
                    eprintln!("{}", err);
                    continue;
                }
            };
            if !dent.file_type().is_file() {
                continue;
            }
            let result = searcher.search_path(
                &matcher,
                dent.path(),
                printer.sink_with_path(&matcher, dent.path()),
            );
            
            if let Err(err) = result {
                eprintln!("{}: {}", dent.path().display(), err);
            }
        }
    }

    // let output = String::from_utf8(printer.into_inner().into_inner())?;
    // println!("OUTPUT: {:?}", output);

    Ok(())
}