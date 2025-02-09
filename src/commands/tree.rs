//
//  Copyright (C) 2022-2025  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use super::plan::PlanError;
use crate::commands::helps::tree;
use crate::commands::plan::Plan;
use crate::core::algo;
use crate::core::algo::IpFileNode;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::lang::node::HdlNode;
use crate::core::lang::node::HdlSymbol;
use crate::core::lang::node::IdentifierFormat;
use crate::core::lang::node::SubUnitNode;
use crate::core::lang::reference::CompoundIdentifier;
use crate::core::lang::vhdl::token::Identifier as VhdlIdentifier;
use crate::core::lang::Lang;
use crate::core::lang::LangIdentifier;
use crate::error::Error;
use crate::error::Hint;
use crate::util::anyerror::Fault;
use crate::util::graph::EdgeStatus;
use crate::util::graphmap::GraphMap;
use std::collections::HashMap;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Tree {
    roots: Option<Vec<VhdlIdentifier>>,
    // compress: bool,
    format: Option<IdentifierFormat>,
    ascii: bool,
    ip: bool,
}

impl Subcommand<Context> for Tree {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(tree::HELP))?;
        Ok(Tree {
            // TODO: implement compression logic
            // compress: cli.check(Arg::flag("compress"))?,
            ascii: cli.check(Arg::flag("ascii"))?,
            ip: cli.check(Arg::flag("ip"))?,
            format: cli.get(Arg::option("format").value("fmt"))?,
            roots: cli.get_all(Arg::positional("unit"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // go to the ip directory
        c.jump_to_working_ip()?;

        // get the ip manifest
        let ip = Ip::load(c.get_ip_path().unwrap().clone(), true, false)?;

        // gather the catalog
        let catalog = Catalog::new().installations(c.get_cache_path())?;

        self.run(ip, catalog)
    }
}

impl Tree {
    fn run(&self, target: Ip, catalog: Catalog) -> Result<(), Fault> {
        match &self.ip {
            true => self.run_ip_graph(target, catalog),
            false => self.run_hdl_graph(target, catalog),
        }
    }

    /// Construct and print the graph at an HDL-entity level.
    fn run_hdl_graph(&self, target: Ip, catalog: Catalog) -> Result<(), Fault> {
        let working_lib = target.get_hdl_library();

        // build graph again but with entire set of all files available from all depdendencies
        let ip_graph = algo::compute_final_ip_graph(&target, &catalog)?;
        let files = algo::build_ip_file_list(&ip_graph, &target);

        // build the complete graph (using entities as the nodes)
        let global_graph = Self::build_graph(&files)?;

        let roots = match &self.roots {
            Some(user_roots) => {
                // restrict graph to units only found within the current IP
                let local_graph = Plan::compute_local_graph(&global_graph, &target);
                let mut roots = Vec::new();
                for root_name in user_roots {
                    // check if the identifier exists in the entity graph
                    let i = match local_graph.get_node_by_key(&&CompoundIdentifier::new(
                        working_lib.clone(),
                        LangIdentifier::Vhdl(root_name.clone()),
                    )) {
                        Some(id) => {
                            // verify the unit is a component
                            if id.as_ref().get_symbol().is_component() == false {
                                return Err(PlanError::BadEntity(root_name.clone()))?;
                            }
                            id.index()
                        }
                        None => {
                            return Err(Error::GetUnitNotFound(
                                root_name.to_string(),
                                Hint::ShowAvailableUnitsLocal,
                            ))?
                        }
                    };
                    roots.push(Plan::local_to_global(i, &global_graph, &local_graph).index())
                }
                roots
            }
            None => {
                // restrict graph to units only found within the current IP
                let local_graph = Plan::compute_local_graph(&global_graph, &target);
                // compile list of all roots
                let mut roots = Vec::new();
                match local_graph.find_root() {
                    Ok(i) => roots.push(
                        Plan::local_to_global(i.index(), &global_graph, &local_graph).index(),
                    ),
                    Err(e) => match e.len() {
                        0 => return Err(PlanError::Empty)?,
                        _ => e.into_iter().for_each(|f| {
                            roots
                                .push(Plan::local_to_global(f, &global_graph, &local_graph).index())
                        }),
                    },
                }
                roots
            }
        };

        // display each root's tree to the console
        roots
            .iter()
            .filter(|k| {
                global_graph
                    .get_node_by_index(**k)
                    .unwrap()
                    .as_ref()
                    .get_symbol()
                    .is_component()
            })
            .for_each(|n| {
                let tree = global_graph.get_graph().treeview(*n);
                for twig in &tree {
                    let branch_str = match self.ascii {
                        true => Self::to_ascii(&twig.0.to_string()),
                        false => twig.0.to_string(),
                    };
                    println!(
                        "{}{}",
                        branch_str,
                        global_graph
                            .get_node_by_index(twig.1)
                            .unwrap()
                            .as_ref()
                            .display(self.format.as_ref().unwrap_or(&IdentifierFormat::Short))
                    );
                }
            });

        Ok(())
    }

    /// Construct and print the graph at an IP dependency level.
    fn run_ip_graph(&self, target: Ip, catalog: Catalog) -> Result<(), Fault> {
        let ip_graph = algo::compute_final_ip_graph(&target, &catalog)?;

        let tree = ip_graph.get_graph().treeview(0);

        for twig in &tree {
            let branch_str = match self.ascii {
                true => Self::to_ascii(&twig.0.to_string()),
                false => twig.0.to_string(),
            };
            println!(
                "{}{}",
                branch_str,
                ip_graph
                    .get_node_by_index(twig.1)
                    .unwrap()
                    .as_ref()
                    .as_ip()
                    .get_man()
                    .get_ip()
                    .into_ip_spec()
            );
        }
        Ok(())
    }

    /// Converts the original treeview text from using extended ascii characters
    /// to orginal ascii characters.
    fn to_ascii(s: &str) -> String {
        let mut transform = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            match c {
                '─' => transform.push('-'),
                '│' => transform.push('|'),
                '├' => transform.push('+'),
                '└' => transform.push('\\'),
                _ => transform.push(c),
            }
        }
        transform
    }

    /// Constructs a graph of the design heirarchy with entity nodes.
    fn build_graph<'a>(
        files: &'a Vec<IpFileNode>,
    ) -> Result<GraphMap<CompoundIdentifier, HdlNode<'a>, ()>, Fault> {
        // entity identifier, HashNode (hash-node holds entity structs)
        let mut graph_map = GraphMap::<CompoundIdentifier, HdlNode, ()>::new();

        let mut sub_nodes: Vec<(LangIdentifier, SubUnitNode)> = Vec::new();
        // store the (suffix, prefix) for all entities
        let mut component_pairs: HashMap<LangIdentifier, LangIdentifier> = HashMap::new();

        // read all files (same as planning)
        for source_file in files {
            match source_file.get_language() {
                Lang::Vhdl => Plan::create_vhdl_node(
                    &mut graph_map,
                    source_file,
                    &mut component_pairs,
                    &mut sub_nodes,
                )?,
                Lang::Verilog => {
                    Plan::create_verilog_node(&mut graph_map, source_file, &mut component_pairs)?
                }
                Lang::SystemVerilog => Plan::create_systemverilog_node(
                    &mut graph_map,
                    source_file,
                    &mut component_pairs,
                )?,
            }
        }

        // differs from planning below

        // add edges according to verilog
        Plan::connect_edges_from_verilog(&mut graph_map, &mut component_pairs, true);

        // go through all subunits and make the connections
        let mut sub_nodes_iter = sub_nodes.into_iter();
        while let Some((lang_lib, node)) = sub_nodes_iter.next() {
            let hdl_lib = lang_lib.as_vhdl_name().unwrap();
            let node_name =
                CompoundIdentifier::new_vhdl(hdl_lib.clone(), node.get_sub().get_entity().clone());

            // link to the owner and add subunit's source file
            // note: this also occurs in `plan.rs`
            let entity_node = match graph_map.get_node_by_key_mut(&node_name) {
                Some(en) => en,
                // @todo: issue error because the entity (owner) is not declared
                None => continue,
            };
            entity_node.as_ref_mut().add_file(node.get_file());
            // create edges by ordered edge list (for entities)
            for dep in node.get_sub().get_edge_list_entities() {
                // need to locate the key with a suffix matching `dep` if it was a component instantiation
                if dep.get_prefix().is_none() {
                    if let Some(lib) = component_pairs.get(dep.get_suffix()) {
                        let b = graph_map.add_edge_by_key(
                            &CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone()),
                            &node_name,
                            (),
                        );
                        match b {
                            // create black box entity
                            EdgeStatus::MissingSource => {
                                let dep_name =
                                    CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone());

                                graph_map.add_node(
                                    dep_name.clone(),
                                    HdlNode::black_box(HdlSymbol::BlackBox(
                                        dep.get_suffix().to_string(),
                                    )),
                                );
                                graph_map.add_edge_by_key(&dep_name, &node_name, ());
                            }
                            _ => (),
                        }
                    // this entity does not exist or was not logged
                    } else {
                        // create new node for black box entity
                        if graph_map.has_node_by_key(dep) == false {
                            graph_map.add_node(
                                dep.clone(),
                                HdlNode::black_box(HdlSymbol::BlackBox(
                                    dep.get_suffix().to_string(),
                                )),
                            );
                        }
                        graph_map.add_edge_by_key(&dep, &node_name, ());
                    }
                // the dependency has a prefix (a library) with it
                } else {
                    graph_map.add_edge_by_key(dep, &node_name, ());
                };
            }
        }

        Ok(graph_map)
    }
}
