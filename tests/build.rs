// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use i_slint_compiler::generator::OutputFormat;
use i_slint_compiler::{
    CompilerConfiguration, compile_syntax_node, diagnostics::BuildDiagnostics, generator, parser,
};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn extract_test_function(source: &str) -> String {
    let mut in_comment = false;
    let mut function = String::new();
    for line in source.lines() {
        if line == "/*" {
            in_comment = true;
            continue;
        }

        if line == "*/" {
            break;
        }

        if !in_comment {
            continue;
        }

        function += line;
        function += "\n";
    }

    function
}

#[derive(Debug)]
struct TestCase {
    source: String,
    name: String,
}

fn main() {
    println!("cargo:rerun-if-changed=../crates/mukwa/ui");
    // TODO: could use async for more performant IO like cargo nextest
    let cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cases");

    let mut test_cases = vec![];
    for entry in fs::read_dir(&cases_dir).unwrap() {
        let entry = entry.unwrap();
        // TODO: add recursive function for dirs

        let file_name = entry.file_name();
        let name: Vec<&str> = file_name.to_str().unwrap().split(".").collect();
        let source = fs::read_to_string(entry.path()).unwrap();
        let case = TestCase {
            source,
            name: String::from(name[0]),
        };
        test_cases.push(case);
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let include_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("crates/mukwa/ui");
    dbg!(&include_path);

    for case in test_cases {
        let mut diag = BuildDiagnostics::default();

        let syntax_node = parser::parse(case.source.clone(), None, &mut diag);
        let library = HashMap::from([("lucide".to_string(), PathBuf::from(lucide_slint::lib()))]);
        let mut compiler_config = CompilerConfiguration::new(OutputFormat::Rust);
        compiler_config.debug_info = true;
        compiler_config.include_paths = vec![include_path.clone()];
        compiler_config.library_paths = library;

        let (root_component, diag, loader) =
            smol::block_on(compile_syntax_node(syntax_node, diag, compiler_config));

        if diag.has_errors() {
            diag.print_warnings_and_exit_on_error();
            // TODO: maybe return an error here
        } else {
            diag.print();
        }

        let mut file = File::create(out_dir.join(format!("{}.rs", case.name))).unwrap();
        generator::generate(
            OutputFormat::Rust,
            &mut file,
            None,
            &root_component,
            &loader.compiler_config,
        )
        .unwrap();

        let test_function = extract_test_function(&case.source);

        write!(
            file,
            "\n#[test]\nfn test_{}(){{\ni_slint_backend_testing::init_no_event_loop();\n{}}}",
            case.name, test_function
        )
        .unwrap();
    }
}
