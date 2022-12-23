use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str;

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

        //TODO: Disregard patterns matched in block comments
        // See this pattern in action: https://regex101.com/r/jd6CX1/4
        let ripgrep_process_output = run_ripgrep(r#"^(#\[cfg\(any\(((?!test, )[ \w\-="]*, )*feature *= *"runtime-benchmarks"((?!, test), [ \w\-="]*)*\)\)\])|^(#\[cfg\(feature *= *"runtime-benchmarks"\)\])"#, ".");
        let ripgrep_output = str::from_utf8(&ripgrep_process_output.stdout).unwrap();

        for line in ripgrep_output.lines() {
    
            if let Ok(json_line) = serde_json::from_str::<MyGrepResult>(line)
                && json_line.type_name == "match"
                && let Some(found_text) = json_line.data["submatches"][0]["match"]["text"].as_str()
                && let Some(found_line) = json_line.data["line_number"].as_u64()
                && let Some(found_file_name) = json_line.data["path"]["text"].as_str() {
            
                let suggested_text: String = create_suggested_text(found_text);
            
                let warning_message = format!("substrace: benchmarks not run in tests.
Found:
{}
at line {} in {}. Suggested replacement:
{}", found_text, found_line, found_file_name, suggested_text);

                //TODO: hir_id does not matter. Isn't there a lint emitter without it? Currently an arbitrary hir_id is used.
                // Emit lint
                cx.tcx.struct_lint_node(ENABLE_SINGLEPASS_BENCHMARKS, cx.last_node_with_lint_attrs, warning_message, |diag| diag)
            }
        }
    }
}

fn create_suggested_text(text: &str) -> String {
    if text.contains("any(") { // Example: #[cfg(any(feature = "runtime-benchmarks"))]
        text.replacen("any(", "any(test, ", 1)
    } else { // Example: #[cfg(feature = "runtime-benchmarks")]
        String::from("#[cfg(any(test, feature = \"runtime-benchmarks\"))]")
    }
}

// Run ripgrep with the supplied pattern and path and output to jsonl format.
fn run_ripgrep(pattern: &str, path: &str) -> std::process::Output {
    Command::new("rg")
        .arg("--json")
        .arg("--pcre2") // necessary for negative look-around functionality
        .arg(pattern)
        .arg(path)
        .output()
        .expect("Failed to run ripgrep.")
}
