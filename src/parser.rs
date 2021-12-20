use std::collections::BTreeMap;

use crate::{
    config::{HoistDeclaration, ProcessArgument, RootMode, ScriptCmd, ScriptDeclaration},
    RootConfiguration,
};
use kdl::{KdlNode, KdlValue};

pub fn parse_kdl_configuration(contents: &str) -> RootConfiguration {
    let document: Vec<KdlNode> = kdl::parse_document(contents).unwrap();

    let mode = document
        .iter()
        .find(|node| node.name.eq("mode"))
        .and_then(|node| {
            let mode = node.values.get(0).unwrap();
            if let KdlValue::String(mode) = mode {
                match mode.as_str() {
                    "standalone" => Some(RootMode::Standalone),
                    _ => todo!(),
                }
            } else {
                None
            }
        })
        .unwrap_or(RootMode::Standalone);

    let hoist = document
        .iter()
        .filter(|node| node.name.eq("hoist"))
        .map(|node| {
            let mut hoist = Vec::new();
            for declaration in node.children.iter() {
                match declaration.name.as_str() {
                    "directory" | "dir" => {
                        let directory = match declaration.values.get(0) {
                            Some(KdlValue::String(name)) => name.clone(),
                            _ => todo!(),
                        };
                        hoist.push(HoistDeclaration::Directory { directory });
                    }
                    _ => todo!(),
                }
            }
            hoist
        })
        .reduce(|mut acc, val| {
            acc.extend(val);
            acc
        })
        .unwrap_or_else(|| Vec::new());

    let scripts = document
        .iter()
        .filter(|node| node.name.eq("scripts"))
        .map(|node| {
            let mut scripts = BTreeMap::new();
            for script_node in node.children.iter().filter(|node| node.name.eq("script")) {
                let name = match script_node.values.get(0) {
                    Some(KdlValue::String(name)) => name.clone(),
                    _ => todo!(),
                };

                let mut steps = Vec::new();
                for step_node in script_node.children.iter() {
                    let cmd = match step_node.values.get(0) {
                        Some(KdlValue::String(name)) => name.clone(),
                        _ => todo!(),
                    };
                    let cwd = step_node.properties.get("cwd").map(|val| match val {
                        KdlValue::String(name) => name.clone(),
                        _ => todo!(),
                    });

                    let mut process_args = BTreeMap::default();
                    for process_arg in step_node
                        .children
                        .iter()
                        .filter(|node| node.name.eq("process-arg"))
                    {
                        let idx = match process_arg.values.get(0) {
                            Some(KdlValue::Int(idx)) => idx.clone(),
                            _ => todo!(),
                        };
                        let idx = u16::try_from(idx).unwrap();
                        let process =
                            if let Some(transform) = process_arg.properties.get("transform") {
                                let transform = match transform {
                                    KdlValue::String(function) => function.clone(),
                                    _ => todo!(),
                                };
                                ProcessArgument::Transform { transform }
                            } else {
                                panic!("unknown or missing process argument property")
                            };
                        process_args.insert(idx, process);
                    }

                    steps.push(ScriptCmd {
                        cmd,
                        cwd,
                        process_args,
                    });
                }

                scripts.insert(name, ScriptDeclaration(steps));
            }
            scripts
        })
        .reduce(|mut acc, val| {
            acc.extend(val);
            acc
        })
        .unwrap_or_else(|| BTreeMap::new());

    RootConfiguration {
        mode,
        hoist,
        scripts,
    }
}
