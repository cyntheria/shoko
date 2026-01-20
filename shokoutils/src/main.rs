use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use shoko::archive::ShokoArchive;
use env_logger;
use log::info;

use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::Direction;

fn main() -> std::io::Result<()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    let mut clevel = 1;
    for arg in &args {
        if arg.starts_with("--clevel=") {
            if let Ok(val) = arg.replace("--clevel=", "").parse::<u8>() {
                clevel = val.clamp(1, 9);
            }
        }
    }

    match args[1].as_str() {
        "pack" => {
            if args.len() < 4 { return print_usage("pack <folder> -o <archive.sk1>"); }
            let folder = &args[2];
            let output = &args[args.len() - 1];
            let mut archive = ShokoArchive::create(output)?;
            pack_recursive(&mut archive, folder, "", clevel)?;
            info!("Packed {} into {} (clevel: {})", folder, output, clevel);
        }
        "unpack" => {
            if args.len() < 3 { return print_usage("unpack <archive.sk1> [out_dir] [--glob=pattern]"); }
            let mut archive = ShokoArchive::open(&args[2])?;
            let out_dir = args.get(3).map(|s| s.as_str()).unwrap_or(".");
            
            let mut target_paths = Vec::new();
            let mut filter = None;
            for arg in &args {
                if arg.starts_with("--glob=") {
                    filter = Some(arg.replace("--glob=", ""));
                }
            }

            if let Some(pattern) = filter {
                target_paths = archive.match_glob(&pattern).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid glob: {}", e))
                })?;
                info!("Glob pattern '{}' matched {} files.", pattern, target_paths.len());
            } else {
                target_paths = archive.entries.iter().map(|e| e.path.clone()).collect();
            }

            fs::create_dir_all(out_dir)?;
            for path_str in target_paths {
                let content = archive.extract_file(&path_str)?;
                let out_path = Path::new(out_dir).join(&path_str);
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(out_path, content)?;
                info!("Extracted: {}", path_str);
            }
            info!("Unpack complete.");
        }
        "read" => {
            if args.len() < 3 { return print_usage("read <archive.sk1>"); }
            let archive = ShokoArchive::open(&args[2])?;
            render_tree(&archive);
        }
        "search" => {
            if args.len() < 4 { return print_usage("search <archive.sk1> <pattern>"); }
            let archive = ShokoArchive::open(&args[2])?;
            let pattern = &args[3];
            let matches = archive.match_glob(pattern).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid glob: {}", e))
            })?;
            
            info!("Matches for '{}':", pattern);
            for m in matches {
                info!("  {}", m);
            }
        }
        "delete" => {
            if args.len() < 4 { return print_usage("delete <archive.sk1> <internal_path>"); }
            let mut archive = ShokoArchive::open(&args[2])?;
            let internal_path = &args[3];
            
            info!("Deleting '{}'...", internal_path);
            archive.delete_file(internal_path)?;
            
            info!("Reclaiming space (defragmenting)...");
            archive.defrag()?;
            info!("Successfully removed and optimized.");
        }
        "write" => {
            if args.len() < 3 { return print_usage("write <archive.sk1>/<file>"); }
            let target = &args[2];
            let (arc_path, inner_path) = target.split_once('/').ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Format: archive.sk1/file.txt")
            })?;

            let mut archive = ShokoArchive::open(arc_path)?;
            let initial_content = archive.extract_file(inner_path).unwrap_or_default();
            
            let tmp_path = ".shoko_edit.tmp";
            fs::write(tmp_path, initial_content)?;
            
            let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            Command::new(editor).arg(tmp_path).status()?;
            
            let new_content = fs::read(tmp_path)?;
            archive.write_file_direct(inner_path, &new_content, clevel)?;
            fs::remove_file(tmp_path)?;
            info!("Successfully updated {}", inner_path);
        }
        _ => print_help(),
    }

    Ok(())
}

fn render_tree(archive: &ShokoArchive) {
    let mut graph = StableGraph::<String, ()>::new();
    let root_idx = graph.add_node("ROOT".to_string());
    
    for entry in &archive.entries {
        let parts: Vec<&str> = entry.path.split('/').collect();
        let mut current_idx = root_idx;
        
        for (i, part) in parts.iter().enumerate() {
            let is_file = i == parts.len() - 1;
            let node_label = if is_file {
                format!("{} ({} bytes)", part, entry.size)
            } else {
                part.to_string()
            };

            let existing = graph.neighbors_directed(current_idx, Direction::Outgoing)
                .find(|&n| {
                    let label = &graph[n];
                    label == part || label.starts_with(&format!("{} (", part))
                });
            
            current_idx = match existing {
                Some(idx) => idx,
                None => {
                    let new_node = graph.add_node(node_label);
                    graph.add_edge(current_idx, new_node, ());
                    new_node
                }
            };
        }
    }

    info!("Archive Structure:");
    print_branch(&graph, root_idx, "");
}

fn print_branch(graph: &StableGraph<String, ()>, node: NodeIndex, prefix: &str) {
    let mut neighbors: Vec<_> = graph.neighbors_directed(node, Direction::Outgoing).collect();
    neighbors.sort_by(|a, b| graph[*a].cmp(&graph[*b]));

    for (i, &child) in neighbors.iter().enumerate() {
        let is_last = i == neighbors.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        println!("{}{}{}", prefix, connector, graph[child]);
        
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        print_branch(graph, child, &new_prefix);
    }
}

fn pack_recursive(archive: &mut ShokoArchive, root: &str, prefix: &str, clevel: u8) -> std::io::Result<()> {
    let root_path = Path::new(root);
    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        let internal_name = if prefix.is_empty() { name.to_string() } else { format!("{}/{}", prefix, name) };

        if path.is_dir() {
            pack_recursive(archive, path.to_str().unwrap(), &internal_name, clevel)?;
        } else {
            let content = fs::read(&path)?;
            archive.write_file_direct(&internal_name, &content, clevel)?;
            info!("Packed: {}", internal_name);
        }
    }
    Ok(())
}

fn print_help() {
    println!("sar - Shoko Archive CLI");
    println!("Commands:");
    println!("  pack <folder> -o <arc>      Create an archive from a folder");
    println!("  unpack <arc> [out]          Extract all files");
    println!("  unpack <arc> --glob='*.txt' Selective extraction");
    println!("  read <arc>                  Show tree structure");
    println!("  search <arc> <glob>         Find files in archive");
    println!("  delete <arc> <path>         Remove file and optimize");
    println!("  write <arc>/<path>          Edit file in-place");
    println!("\nFlags:");
    println!("  --clevel=N (1-9)            Set RLE compression threshold");
}

fn print_usage(s: &str) -> std::io::Result<()> {
    println!("Usage: sar {}", s);
    Ok(())
}
