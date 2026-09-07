// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::io::Write;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use i_slint_compiler::{diagnostics::BuildDiagnostics, parser, CompilerConfiguration, compile_syntax_node, generator};
use i_slint_compiler::{generator::OutputFormat};

fn extract_test_function(source: &str) -> String{
    let mut in_comment = false;
    let mut function = String::new();
    for line in source.lines(){
        if line == "/*"{
            in_comment = true;
            continue;
        }

        if line == "*/"{
            in_comment = false;
            break;
        }

        if !in_comment{
            continue;
        }

        function += line;
        function += "\n";
    }

    function
}

fn main() {
    // TODO: could use async for more performant IO like cargo nextest
    let cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cases");
    let source = fs::read_to_string(cases_dir.join("button.slint")).unwrap();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    dbg!(&out_dir);

    let mut diag = BuildDiagnostics::default();
    let syntax_node = parser::parse(source.clone(),None,&mut diag);
    let mut compiler_config = CompilerConfiguration::new(OutputFormat::Rust);
    compiler_config.debug_info = true;

    let (root_component,diag,loader) = smol::block_on(compile_syntax_node(syntax_node,diag,compiler_config));

    if diag.has_errors(){
        diag.print_warnings_and_exit_on_error();
        // TODO: maybe return an error here
    } else {
        diag.print();
    }

    let mut file = File::create(out_dir.join("test.rs")).unwrap();
    //let mut file = File::create("test.rs").unwrap();
    //let mut output = Vec::new();
    generator::generate(OutputFormat::Rust,&mut file,None,&root_component,&loader.compiler_config).unwrap();
    //dbg!(output);

    let test_function = extract_test_function(&source);
    dbg!(&test_function);

    write!(file,"\n#[test]\nfn test_button(){{\ni_slint_backend_testing::init_no_event_loop();\n{}}}",test_function).unwrap();
    //panic!();
}

