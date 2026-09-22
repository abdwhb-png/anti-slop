use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{self, BufRead},
    path::Path,
    time::Instant,
};

fn analyze(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let source = fs::read_to_string(path)?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::from_path(Path::new(path))?).parse();
    let parse_errors: Vec<_> = parsed
        .diagnostics
        .iter()
        .map(|e| format!("{e:?}"))
        .collect();
    if !parse_errors.is_empty() {
        return Ok(json!({"file":path,"parse_errors":parse_errors,"status":"parse_failed"}));
    }
    let built = SemanticBuilder::new_compiler()
        .with_build_nodes(true)
        .build(&parsed.program);
    let semantic = &built.semantic;
    let scoping = semantic.scoping();
    let symbols: Vec<_> = scoping.symbol_ids().map(|id| {
        let span = scoping.symbol_span(id);
        let references: Vec<_> = semantic.symbol_references(id).map(|r| {
            let span = semantic.nodes().get_node(r.node_id()).span();
            json!({"start":span.start,"end":span.end,"text":&source[span.start as usize..span.end as usize],"flags":format!("{:?}",r.flags())})
        }).collect();
        json!({"name":scoping.symbol_name(id),"start":span.start,"end":span.end,"flags":format!("{:?}",scoping.symbol_flags(id)),"references":references})
    }).collect();
    let nodes: Vec<_> = semantic.nodes().iter().filter_map(|node| {
        let kind = match node.kind() {
            AstKind::ImportDeclaration(_) => "import",
            AstKind::ImportExpression(_) => "dynamic_import",
            AstKind::CallExpression(_) => "call",
            AstKind::TSTypeAliasDeclaration(_) => "type_alias",
            AstKind::TSInterfaceDeclaration(_) => "interface",
            AstKind::Function(_) => "function",
            _ => return None,
        };
        let span=node.span();
        Some(json!({"kind":kind,"start":span.start,"end":span.end,"text":&source[span.start as usize..span.end as usize]}))
    }).collect();
    let comments: Vec<_> = semantic
        .comments()
        .iter()
        .map(|c| json!({"start":c.span.start,"end":c.span.end}))
        .collect();
    let unresolved: Vec<_> = scoping
        .root_unresolved_references()
        .keys()
        .map(|s| s.as_str())
        .collect();
    Ok(
        json!({"file":path,"bytes":source.len(),"status":"local_facts_only","parse_errors":parse_errors,"semantic_errors":built.diagnostics.iter().map(|e|format!("{e:?}")).collect::<Vec<_>>(),"symbols":symbols,"nodes":nodes,"comments":comments,"unresolved_local_names":unresolved}),
    )
}

fn run(paths: &[String]) -> Value {
    let start = Instant::now();
    let files: Vec<_> = paths
        .iter()
        .map(|p| {
            analyze(p).unwrap_or_else(|e| json!({"file":p,"status":"failed","error":e.to_string()}))
        })
        .collect();
    json!({"analyzer":"oxc","version":"0.150.0","scope":"per_file_syntax_and_local_bindings","files":files,"analysis_ms":start.elapsed().as_secs_f64()*1000.0})
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--server") {
        for line in io::stdin().lock().lines() {
            let paths: Vec<String> = serde_json::from_str(&line?)?;
            println!("{}", run(&paths));
        }
    } else {
        println!("{}", run(&args));
    }
    Ok(())
}
