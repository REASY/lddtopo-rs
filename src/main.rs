mod id_gen;

use clap::Parser;

use crate::id_gen::IdGen;

use lddtree::{DependencyAnalyzer, DependencyTree};

use petgraph::algo::toposort;
use petgraph::graphmap::DiGraphMap;
use petgraph::dot::{Dot, Config};

use serde::{Serialize, Deserialize};
use serde_json;

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::{Path, PathBuf};

#[macro_use]
extern crate log;

use log::info;
use petgraph::Graph;
use petgraph::graph::NodeIndex;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to shared library to analyze
    #[clap(long)]
    shared_library_path: PathBuf,

    /// Root path
    #[clap(long)]
    root_path: Option<PathBuf>,

    /// Additional library paths are treated as absolute paths, not relative to root
    #[clap(long)]
    library_paths: Option<Vec<PathBuf>>,

    /// The path to output file with topologically sorted dependency graph
    #[clap(long)]
    output_file: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, PartialOrd, Ord, PartialEq, Eq)]
struct Edge {
    src: String,
    dst: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Lib {
    name: String,
    path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TopoSortResult {
    vertices: Vec<String>,
    edges: Vec<Edge>,
    library_map: BTreeMap<String, Lib>,
    topo_sorted_libs: Vec<Lib>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    assert!(args.shared_library_path.exists(), "Provided shared library at {} does not exist", args.shared_library_path.to_str().unwrap());

    let root = args.root_path.unwrap_or(PathBuf::from("/"));
    let analyzer = match args.library_paths {
        None => DependencyAnalyzer::new(root),
        Some(library_paths) => DependencyAnalyzer::new(root).library_paths(library_paths),
    };
    let main_file_name = String::from(args.shared_library_path.file_name().unwrap().to_str().unwrap());
    let main_file_path = String::from(args.shared_library_path.to_str().unwrap());

    let deps: DependencyTree = analyzer.analyze(args.shared_library_path).unwrap();
    info!("{} has {} dependencies", main_file_name, deps.libraries.len());

    let result = get_topologically_sorted_result(&main_file_name, &main_file_path, &deps);
    serde_json::to_writer_pretty(&File::create(args.output_file.clone()).unwrap(), &result).unwrap();

    let dot_path = Path::new(&args.output_file).parent().unwrap().join(format!("{}.dot", Path::new(&args.output_file).file_stem().unwrap().to_str().unwrap()));
    export_to_dot(&result, dot_path);
}

fn export_to_dot(result: &TopoSortResult, dot_path: PathBuf) {
    let mut graph_to_export = Graph::<_, i32>::new();
    let mut vertex_to_index: HashMap::<String, NodeIndex> = HashMap::new();
    result.vertices.iter().for_each(|v| {
        let idx: NodeIndex = graph_to_export.add_node(v.clone());
        vertex_to_index.insert(v.clone(), idx);
    });
    result.edges.iter().for_each(|edge| {
        let from_idx = vertex_to_index.get(&edge.src).unwrap().clone();
        let to_idx = vertex_to_index.get(&edge.dst).unwrap().clone();
        graph_to_export.add_edge(from_idx, to_idx, 0);
    });
    std::fs::write(dot_path, format!("{}", Dot::with_config(&graph_to_export, &[Config::EdgeNoLabel])))
        .expect("Unable to write file");
}

fn get_topologically_sorted_result(main_lib_name: &str, main_lib_path: &str, deps: &DependencyTree) -> TopoSortResult {
    // Imagine we have 6 libraries, A, B, C, D, E and F
    // A depends on B
    // A depends on C
    // A depends on F
    // B depends on D
    // C depends on D
    // D depends on E
    // E depends on F
    // The following direct acyclic graph represents the dependency between libraries, the edge means `depends`, A -> B means A depends on B
    /*
          ┌─────────────┐
          │             │
   ┌──────A──────┐      │
   │             │      │
   │             │      │
   ▼             ▼      │
   B             C      │
   │             │      │
   └─────►D◄─────┘      │
          │             │
          │             │
          ▼             ▼
          E───────────► F
    */
    // The usage of topological sorting from Wiki:
    // The canonical application of topological sorting is in scheduling a sequence of jobs or tasks based on their dependencies.
    // The jobs are represented by vertices, and there is an edge from x to y if job x must be completed before job y can be started

    // If library A depends on library B, B must come before A (B must be loaded first).
    // In terms of DAG it means we should swap the edge between vertices, the graph will become
    /*

  ┌──────F───────┐
  │              │
  ▼              ▼
  E       ┌─────►A◄─────┐
  │       │             │
  │       B             C
  │       ▲             ▲
  │       └──────D──────┘
  │              ▲
  └──────────────┘
     */

    let mut di_graph_map = DiGraphMap::new();
    let mut id_gen = IdGen::new();

    let main_lib_id: u32 = id_gen.get_next_id(main_lib_name);
    for direct_dep in &deps.needed {
        let direct_lib_id = id_gen.get_next_id(direct_dep.as_str());
        if !di_graph_map.contains_node(direct_lib_id) {
            di_graph_map.add_node(direct_lib_id);
        }
        // `main_lib_id` depends on `direct_lib_id`, but the edge points that `direct_lib_id` must come before `main_lib_id`
        di_graph_map.add_edge(direct_lib_id, main_lib_id, ());
    }

    for (_, lib) in &deps.libraries {
        let lib_id = id_gen.get_next_id(lib.name.as_str());
        if !di_graph_map.contains_node(lib_id) {
            di_graph_map.add_node(lib_id);
        }
        for needed in &lib.needed {
            if let Some(dep_lib) = deps.libraries.get(needed) {
                let dep_lib_id = id_gen.get_next_id(dep_lib.name.as_str());
                if !di_graph_map.contains_node(dep_lib_id) {
                    di_graph_map.add_node(dep_lib_id);
                }
                // `lib_id` depends on `dep_lib_id`, but the edge points that `dep_lib_id` must come before `lib_id`
                di_graph_map.add_edge(dep_lib_id, lib_id, ());
            }
        }
    }
    let mut vertices: Vec<String> = Vec::with_capacity(di_graph_map.node_count());
    di_graph_map.nodes().for_each(|vertex_id| {
        let v = String::from(id_gen.get_by_id(vertex_id).unwrap());
        vertices.push(v.clone());
    });
    vertices.sort();

    let mut edges: Vec<Edge> = Vec::with_capacity(di_graph_map.edge_count());
    di_graph_map.all_edges().for_each(|(from, to, _)| {
        let from = String::from(id_gen.get_by_id(from).unwrap());
        let to = String::from(id_gen.get_by_id(to).unwrap());
        edges.push(Edge { src: from, dst: to });
    });
    edges.sort();

    let mut library_map: BTreeMap<String, Lib> = BTreeMap::new();
    for (name, lib) in &deps.libraries {
        let path = String::from(lib.path.as_path().to_str().unwrap());
        library_map.insert(name.clone(), Lib { name: name.clone(), path: Some(path) });
    }

    let topological_sorted = toposort(&di_graph_map, None).unwrap();
    let mut topo_sorted_libs: Vec<Lib> = Vec::with_capacity(topological_sorted.len());
    for id in &topological_sorted {
        let lib_name = id_gen.get_by_id(*id).unwrap();
        let lib_path = if lib_name != main_lib_name {
            deps.libraries.get(lib_name).map(|lib| {
                String::from(lib.path.clone().as_path().to_str().unwrap())
            })
        } else { Some(String::from(main_lib_path)) };
        topo_sorted_libs.push(Lib {
            name: String::from(lib_name),
            path: lib_path,
        });
    }
    return TopoSortResult {
        vertices: vertices,
        edges: edges,
        library_map: library_map,
        topo_sorted_libs: topo_sorted_libs,
    };
}

