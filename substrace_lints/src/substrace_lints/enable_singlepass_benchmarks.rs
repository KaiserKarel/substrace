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
use std::process::Command;
use std::str;

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

//TODO: Implement auto-fix.
impl<'tcx> LateLintPass<'tcx> for EnableSinglepassBenchmarks {

    fn check_crate(&mut self, cx: &LateContext<'tcx>) {

        let ripgrep_process_output = run_ripgrep(r#"^#\[cfg\(feature *= *"runtime-benchmarks"\)\]"#, ".");
        let ripgrep_output = str::from_utf8(&ripgrep_process_output.stdout).unwrap();

        for line in ripgrep_output.lines() {
    
            if let Ok(json_line) = serde_json::from_str::<MyGrepResult>(&line)
                && json_line.type_name == "match"
                && let Some(found_text) = json_line.data["submatches"][0]["match"]["text"].as_str()
                && let Some(found_line) = json_line.data["line_number"].as_u64()
                && let Some(found_file_name) = json_line.data["path"]["text"].as_str() {
            
                let suggested_text: &str = "#[cfg(any(feature = \"runtime-benchmarks\", test))]";
            
                let warning_message = format!("substrace: benchmarks not run in tests.
Found:
{}
at line {} in {}. Suggested replacement:
{}", found_text, found_line, found_file_name, suggested_text);

                //TODO: hir_id does not matter. Isn't there a lint emitter without it? Currently I grab a random one.
                // Emit lint
                cx.tcx.struct_lint_node(ENABLE_SINGLEPASS_BENCHMARKS, cx.last_node_with_lint_attrs, warning_message, |diag| diag)
            }
        }
    }
}

// Run ripgrep with the supplied pattern and path and output to jsonl format.
fn run_ripgrep(pattern: &str, path: &str) -> std::process::Output {
    Command::new("rg")
        .arg("--json")
        .arg(pattern)
        .arg(path)
        .output()
        .expect("Failed to run ripgrep.")
}
