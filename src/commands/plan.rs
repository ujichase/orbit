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

use crate::commands::download::Download;
use crate::core::blueprint::{Blueprint, Instruction, Scheme};
use crate::core::context::{self, Context};
use crate::core::fileset::Fileset;
use crate::core::iparchive::IpArchive;
use crate::core::lang::parser::ParseError;
use crate::core::lang::reference::CompoundIdentifier;
use crate::core::lang::sv::symbols::{SystemVerilogParser, SystemVerilogSymbol};
use crate::core::lang::verilog::symbols::{VerilogParser, VerilogSymbol};
use crate::core::lang::vhdl::subunit::SubUnit;
use crate::core::lang::vhdl::symbols::{VHDLParser, VhdlSymbol};
use crate::core::lang::vhdl::token::Identifier;
use crate::core::lang::{self, Lang, LangIdentifier};
use crate::core::swap;
use crate::core::swap::StrSwapTable;
use crate::core::target::Target;
use crate::core::version::AnyVersion;
use crate::error::{Error, Hint, LastError};
use crate::util::anyerror::Fault;
use crate::util::environment;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::util::filesystem;
use crate::util::graph::EdgeStatus;
use crate::util::graphmap::GraphMap;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};

use crate::commands::install::Install;
use crate::core::algo;
use crate::core::algo::IpFileNode;
use crate::core::algo::IpNode;
use crate::core::catalog::Catalog;
use crate::core::ip::Ip;
use crate::core::ip::IpSpec;
use crate::core::lockfile::LockEntry;
use crate::core::lockfile::LockFile;
use crate::util::graphmap::Node;

#[derive(Debug, PartialEq)]
pub struct Plan {
    target: Option<String>,
    bench: Option<Identifier>,
    top: Option<Identifier>,
    clean: bool,
    list: bool,
    all: bool,
    target_dir: Option<String>,
    filesets: Option<Vec<Fileset>>,
    only_lock: bool,
    force: bool,
}

impl Plan {
    /// Performs the backend logic for creating a blueprint file (planning a design).
    ///
    /// If a blueprint was created, it will return the file name for that blueprint.
    pub fn run(
        working_ip: &Ip,
        target_dir: &str,
        target: &Target,
        catalog: Catalog,
        clean: bool,
        force: bool,
        only_lock: bool,
        all: bool,
        bench_name: &Option<Identifier>,
        top_name: &Option<Identifier>,
        filesets: &Option<Vec<Fileset>>,
        scheme: &Scheme,
        require_bench: bool,
        allow_bench: bool,
        envs: Environment,
    ) -> Result<Option<String>, Fault> {
        // create the output path to know where to begin storing files
        let working_ip_path = working_ip.get_root().clone();
        let target_path = working_ip_path.join(target_dir);
        let output_path = target_path.join(target.get_name());

        // build entire ip graph and resolve with dynamic symbol transformation
        let ip_graph = match algo::compute_final_ip_graph(&working_ip, &catalog) {
            Ok(g) => g,
            Err(e) => {
                // generate a single blueprint
                if e.is_source_err() == true && force == true {
                    let mut blueprint = Blueprint::new(scheme.clone());
                    let ip_file_node = IpFileNode::new(
                        e.as_source_file().unwrap().to_string(),
                        &working_ip,
                        LangIdentifier::new_working(),
                    );
                    blueprint.add(Instruction::Hdl(&ip_file_node));

                    let blueprint_name = blueprint.get_filename();
                    let blueprint_path = Self::create_outputs(
                        &blueprint,
                        envs,
                        &target_path,
                        &String::new(),
                        &String::new(),
                        &String::new(),
                        &String::new(),
                        &String::new(),
                        &String::new(),
                        target,
                        require_bench,
                    )?;
                    // create a blueprint file
                    crate::warn!(
                        "erroneous blueprint created at {:?}",
                        filesystem::into_std_str(blueprint_path)
                    );
                    return Ok(Some(blueprint_name));
                } else {
                    return match e.is_source_err() {
                        true => Err(Error::SourceCodeInvalidSyntax(
                            e.as_source_file().unwrap().clone().into(),
                            LastError(e.into_fault().to_string()),
                        ))?,
                        false => Err(Error::IpGraphFailed(LastError(e.into_fault().to_string())))?,
                    };
                }
            }
        };

        // only write lockfile and exit if flag is raised
        if only_lock == true {
            Self::write_lockfile(&working_ip, &ip_graph, force, true, &catalog)?;
            return Ok(None);
        }

        // check if to clean the directory
        if clean == true && Path::exists(&output_path) == true {
            fs::remove_dir_all(&output_path)?;
        }

        let files = algo::build_ip_file_list(&ip_graph, &working_ip);

        let global_graph = Self::build_full_graph(&files)?;

        let working_lib = working_ip.get_hdl_library();

        // restrict graph to units only found within the current IP
        let local_graph: GraphMap<&CompoundIdentifier, &HdlNode, &()> =
            Self::compute_local_graph(&global_graph, &working_ip);

        let (top, bench) = match allow_bench {
            true => {
                match Self::detect_bench(
                    &global_graph,
                    &local_graph,
                    &working_lib,
                    &bench_name,
                    &top_name,
                ) {
                    Ok(r) => r,
                    Err(e) => match e {
                        PlanError::Ambiguous(_, _, _) => {
                            if all == true {
                                (None, None)
                            } else {
                                return Err(e)?;
                            }
                        }
                        _ => return Err(e)?,
                    },
                }
            }
            false => (None, None),
        };

        // determine the top-level node index
        let (top, bench) = match Self::detect_top(
            &global_graph,
            &local_graph,
            &working_lib,
            top,
            bench,
            &top_name,
            allow_bench,
        ) {
            Ok(r) => r,
            Err(e) => match e {
                PlanError::Ambiguous(_, _, _) => {
                    if all == true {
                        (top, bench)
                    } else {
                        return Err(e)?;
                    }
                }
                PlanError::TestbenchNoTest(_) => (None, bench),
                _ => return Err(e)?,
            },
        };

        let top = match top {
            Some(i) => Some(Self::local_to_global(i, &global_graph, &local_graph).index()),
            None => None,
        };

        let bench = match bench {
            Some(i) => Some(Self::local_to_global(i, &global_graph, &local_graph).index()),
            None => None,
        };
        // guarantees top exists if not using --all

        // error if the user-defined top is not instantiated in the testbench. Say this can be fixed by adding '--all'
        if let Some(b) = &bench {
            // @idea: merge two topological sorted lists together by running top sort from bench and top sort from top if in this situation
            if all == false
                && top.is_some()
                && global_graph
                    .get_graph()
                    .successors(top.unwrap())
                    .find(|i| i == b)
                    .is_none()
            {
                let given_top = global_graph
                    .get_key_by_index(top.unwrap())
                    .unwrap()
                    .get_suffix();
                let given_bench = global_graph.get_key_by_index(*b).unwrap().get_suffix();
                return Err(Error::TopNotInTestbench(
                    given_top.clone(),
                    given_bench.clone(),
                    Hint::IncludeAllInPlan,
                ))?;
            }
        } else if bench.is_none() == true && require_bench == true {
            return Err(Error::TestbenchRequired)?;
        }

        // [!] write the lock file
        Self::write_lockfile(&working_ip, &ip_graph, true, true, &catalog)?;

        // compute minimal topological ordering
        let min_order = match all {
            // perform topological sort on the entire graph
            true => {
                match local_graph.find_root() {
                    // only one topological sorting to compute
                    Ok(r) => {
                        let id = Self::local_to_global(r.index(), &global_graph, &local_graph);
                        global_graph
                            .get_graph()
                            .minimal_topological_sort(id.index())
                    }
                    // exclude roots that do not belong to the local graph
                    Err(roots) => {
                        // println!("{:?}", roots);
                        let mut order = Vec::new();
                        // create dummy node to rely on all known roots
                        roots.iter().for_each(|r| {
                            let id = Self::local_to_global(*r, &global_graph, &local_graph);
                            let mut subset = global_graph
                                .get_graph()
                                .minimal_topological_sort(id.index());
                            order.append(&mut subset);
                        });
                        order
                    }
                }
            }
            // perform topological sort on minimal subset of the graph
            false => {
                // determine which point is the upmost root
                let highest_point = match bench {
                    Some(b) => b,
                    None => match top {
                        Some(t) => t,
                        None => return Err(AnyError(format!("no top-level unit exists")))?,
                    },
                };
                global_graph
                    .get_graph()
                    .minimal_topological_sort(highest_point)
            }
        };

        // println!("{:?}", min_order);

        // generate the file order while merging dependencies for common file path names together
        let file_order = Self::determine_file_order(&global_graph, min_order);

        // remove duplicate files from list while perserving order
        let file_order = Self::remove_multi_occurences(&file_order);

        // grab the names as strings
        let top_name = match top {
            Some(i) => global_graph
                .get_key_by_index(i)
                .unwrap()
                .get_suffix()
                .to_string(),
            None => String::new(),
        };
        let bench_name = match bench {
            Some(i) => global_graph
                .get_key_by_index(i)
                .unwrap()
                .get_suffix()
                .to_string(),
            None => String::new(),
        };

        // grab the files that contain the top-level/dut design unit
        let top_file = match top {
            Some(i) => {
                let n = global_graph.get_node_by_index(i).unwrap();
                n.as_ref()
                    .get_associated_files()
                    .first()
                    .unwrap()
                    .get_file()
            }
            None => "",
        };

        // grab the json representation of the top level unit
        let top_json = match top {
            Some(i) => {
                let n = global_graph.get_node_by_index(i).unwrap();
                let sym = n.as_ref().get_symbol();
                if let Some(m) = sym.as_module() {
                    serde_json::to_string(m)?
                } else if let Some(e) = sym.as_entity() {
                    serde_json::to_string(e)?
                } else {
                    String::new()
                }
            }
            None => String::new(),
        };

        // grab the file that contains the testbench unit
        let bench_file = match bench {
            Some(i) => {
                let n = global_graph.get_node_by_index(i).unwrap();
                n.as_ref()
                    .get_associated_files()
                    .first()
                    .unwrap()
                    .get_file()
            }
            None => "",
        };

        // grab the json representation of the testbench unit
        let bench_json = match bench {
            Some(i) => {
                let n = global_graph.get_node_by_index(i).unwrap();
                let sym = n.as_ref().get_symbol();
                if let Some(m) = sym.as_module() {
                    serde_json::to_string(m)?
                } else if let Some(e) = sym.as_entity() {
                    serde_json::to_string(e)?
                } else {
                    String::new()
                }
            }
            None => String::new(),
        };

        // print information (maybe also print the plugin saved to .env too?)
        match top_name.is_empty() {
            false => match require_bench {
                true => crate::info!("dut set to {}", top_name.blue()),
                false => crate::info!("p-level set to {}", top_name.blue()),
            },
            true => match require_bench {
                true => crate::warn!("no dut set"),
                false => crate::warn!("no top-level set"),
            },
        }
        if require_bench == true {
            match bench_name.is_empty() {
                false => crate::info!("testbench set to {}", bench_name.blue()),
                true => crate::warn!("no testbench set"),
            }
        }

        // store data in blueprint
        let mut blueprint = Blueprint::new(scheme.clone());

        // [!] collect user-defined filesets
        {
            let current_files: Vec<String> = working_ip.gather_current_files();

            let mut vtable = StrSwapTable::new();
            // variables could potentially store empty strings if units are not set
            vtable.add("orbit.tb.name", &bench_name);
            vtable.add("orbit.top.name", &top_name);
            vtable.add("orbit.dut.name", &top_name);

            // store data in a map for quicker look-ups when comparing to plugin-defind filesets
            let mut cli_fset_map: HashMap<&String, &Fileset> = HashMap::new();

            // use command-line set filesets
            if let Some(fsets) = filesets {
                for fset in fsets {
                    // insert into map structure
                    cli_fset_map.insert(fset.get_name(), &fset);
                }
            }

            // collect data for the given plugin
            if let Some(filesets) = target.get_filesets() {
                for (name, pattern) in filesets {
                    let proper_key = Fileset::standardize_name(name);
                    // check if appeared in cli arguments
                    let (f_name, f_patt) = match cli_fset_map.contains_key(&proper_key) {
                        // override with fileset provided by command-line if conflicting names
                        true => {
                            // pull from map to ensure it is not double-counted when just writing command-line filesets
                            let entry = cli_fset_map.remove(&proper_key);
                            (name, entry.unwrap().get_pattern())
                        }
                        false => (name, pattern.inner()),
                    };
                    // perform variable substitution
                    let fset = Fileset::new()
                        .name(f_name)
                        .pattern(&swap::substitute(f_patt.to_string(), &vtable))?;
                    // match files
                    fset.collect_files(&current_files)
                        .into_iter()
                        .for_each(|f| {
                            blueprint.add(Instruction::Auxiliary(
                                fset.get_name().clone(),
                                working_lib.to_string(),
                                f.clone(),
                            ));
                        });
                }
            }

            // check against every defined fileset in the command-line (call remaining filesets)
            for (_key, fset) in cli_fset_map {
                // perform variable substitution
                let fset = Fileset::new()
                    .name(fset.get_name())
                    .pattern(&swap::substitute(fset.get_pattern().to_string(), &vtable))?;
                // match files
                fset.collect_files(&current_files)
                    .into_iter()
                    .for_each(|f| {
                        blueprint.add(Instruction::Auxiliary(
                            fset.get_name().clone(),
                            working_lib.to_string(),
                            f.clone(),
                        ));
                    });
            }
        }

        // collect in-order HDL file list
        for ip_file_node in file_order {
            if fileset::is_rtl(&ip_file_node.get_file()) == true {
                blueprint.add(Instruction::Hdl(ip_file_node));
            } else {
                blueprint.add(Instruction::Hdl(ip_file_node));
            }
        }

        let blueprint_name = blueprint.get_filename();

        let blueprint_path = Self::create_outputs(
            &blueprint,
            envs,
            &target_path,
            &top_name,
            &top_file,
            &top_json,
            &bench_name,
            &bench_file,
            &bench_json,
            target,
            require_bench,
        )?;
        // create a blueprint file
        crate::info!(
            "blueprint created at: {:?}",
            filesystem::into_std_str(blueprint_path)
        );
        Ok(Some(blueprint_name))
    }
}

pub fn resolve_missing_deps<'a>(
    c: &'a Context,
    working_ip: &'a Ip,
    mut catalog: Catalog<'a>,
    force: bool,
) -> Result<Catalog<'a>, Fault> {
    // this code is only ran if the lock file matches the manifest and we aren't force to recompute
    if working_ip.can_use_lock(&catalog) == true && force == false {
        let le: LockEntry = LockEntry::from((working_ip, true));
        let lf = working_ip.get_lock();

        let env = Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?;
        let vtable = StrSwapTable::new().load_environment(&env)?;

        download_missing_deps(vtable, &lf, &le, &catalog, &c.get_config().get_protocols())?;
        // recollect the downloaded items to update the catalog for installations
        catalog = catalog.downloads(c.get_downloads_path())?;

        install_missing_deps(&lf, &le, &catalog)?;
        // recollect the installations to update the catalog for dependency graphing
        catalog.installations(c.get_cache_path())
    } else {
        Ok(catalog)
    }
}

pub fn download_missing_deps(
    vtable: StrSwapTable,
    lf: &LockFile,
    le: &LockEntry,
    catalog: &Catalog,
    protocols: &ProtocolMap,
) -> Result<(), Fault> {
    let mut vtable = vtable;
    // fetch all non-downloaded packages
    for entry in lf.inner() {
        // skip the current project's IP entry or any IP already in the downloads/
        if entry.matches_target(le, &catalog) == true
            || catalog.is_downloaded_slot(&entry.to_download_slot_key()) == true
            || entry.is_relative() == true
        {
            continue;
        }

        let ver = AnyVersion::Specific(entry.get_version().to_partial_version());

        let mut require_download = false;

        match catalog.inner().get(entry.get_uuid()) {
            Some(status) => {
                match status.get_install(&ver) {
                    Some(dep) => {
                        // verify the checksum
                        if Install::is_checksum_good(&dep.get_root()) == false {
                            crate::info!(
                                "redownloading ip {} due to bad checksum ...",
                                dep.get_man().get_ip().into_ip_spec()
                            );
                            require_download = true;
                        }
                    }
                    None => {
                        match status.get_download(&ver) {
                            // already exists in the downloads
                            Some(_) => (),
                            // does not exist in the downloads
                            None => {
                                require_download = true;
                            }
                        }
                    }
                }
            }
            // does not exist at all in the catalog
            None => {
                require_download = true;
            }
        }
        // check if the slot is not already filled before trying to download
        if require_download == true {
            match entry.get_source() {
                Some(src) => {
                    // fetch from the internet
                    Download::download(
                        &mut vtable,
                        Some(&entry.to_ip_spec().to_partial_ip_spec()),
                        src,
                        None,
                        catalog.get_downloads_path(),
                        &protocols,
                        false,
                        true,
                    )?;
                }
                None => {
                    return Err(AnyError(format!(
                        "unable to fetch ip {} from the internet due to missing source",
                        entry.to_ip_spec()
                    )))?;
                }
            }
        }
    }
    Ok(())
}

pub fn install_missing_deps(lf: &LockFile, le: &LockEntry, catalog: &Catalog) -> Result<(), Fault> {
    // fill in the catalog with missing modules according the lock file if available
    for entry in lf.inner() {
        // skip the current project's IP entry
        if entry.matches_target(&le, &catalog) {
            continue;
        }

        let ver = AnyVersion::Specific(entry.get_version().to_partial_version());

        // try to use the lock file to fill in missing pieces
        match catalog.inner().get(entry.get_uuid()) {
            Some(status) => {
                // find this IP to read its dependencies
                match status.get_install(&ver) {
                    // no action required (already installed)
                    Some(dep) => {
                        // verify the checksum in case we need to re-install from downloads
                        if Install::is_checksum_good(&dep.get_root()) == false {
                            match status.get_download(&ver) {
                                Some(dep) => {
                                    crate::info!(
                                        "reinstalling ip {} due to bad checksum ...",
                                        dep.get_man().get_ip().into_ip_spec()
                                    );
                                    // perform extra work if the Ip is virtual (from downloads)
                                    install_ip_from_downloads(&dep, &catalog, true)?
                                }
                                None => {
                                    // failed to get the install from the queue
                                    return Err(Box::new(Error::EntryMissingDownload(
                                        entry.to_ip_spec(),
                                    )));
                                }
                            }
                        }
                    }
                    // install
                    None => {
                        // check the queue for installation
                        match status.get_download(&ver) {
                            Some(dep) => {
                                // perform extra work if the Ip is virtual (from downloads)
                                install_ip_from_downloads(&dep, &catalog, false)?
                            }
                            None => {
                                return Err(Box::new(Error::EntryNotQueued(entry.to_ip_spec())))
                            }
                        }
                    }
                }
            }
            None => {
                // check if its a relative ip
                if entry.is_relative() == false {
                    return Err(Box::new(Error::EntryUnknownIp(entry.to_ip_spec())));
                }
            }
        }
    }
    Ok(())
}

fn install_ip_from_downloads(dep: &Ip, catalog: &Catalog, force: bool) -> Result<(), Fault> {
    // perform extra work if the Ip is virtual (from downloads)
    if let Some(bytes) = dep.get_mapping().as_bytes() {
        // place the dependency into a temporary directory
        let dir = tempfile::tempdir()?.into_path();
        if let Err(e) = IpArchive::extract(&bytes, &dir) {
            fs::remove_dir_all(dir)?;
            return Err(e);
        }
        // load the IP
        let unzipped_dep = match Ip::load(dir.clone(), false, false) {
            Ok(x) => x,
            Err(e) => {
                fs::remove_dir_all(dir)?;
                return Err(e);
            }
        };
        // install from the unzipp ip
        match Install::install(&unzipped_dep, catalog.get_cache_path(), force, true) {
            Ok(_) => {}
            Err(e) => {
                fs::remove_dir_all(dir)?;
                return Err(e);
            }
        }
        fs::remove_dir_all(unzipped_dep.get_root())?;
    } else {
        panic!("trying to download from a physical path")
    }
    Ok(())
}

use crate::core::fileset;
use crate::util::anyerror::AnyError;

use super::download::ProtocolMap;

use crate::core::lang::node::SubUnitNode;
use crate::core::lang::node::{HdlNode, HdlSymbol};

impl Plan {
    pub fn create_verilog_node<'a, 'b>(
        graph_map: &'b mut GraphMap<CompoundIdentifier, HdlNode<'a>, ()>,
        node: &'a IpFileNode,
        component_pairs: &'b mut HashMap<LangIdentifier, LangIdentifier>,
    ) -> Result<(), Fault> {
        let contents = lang::read_to_string(&node.get_file())?;
        let symbols = match VerilogParser::read(&contents) {
            Ok(s) => s.into_symbols(),
            Err(e) => Err(ParseError::SourceCodeError(
                node.get_file().clone(),
                e.to_string(),
            ))?,
        };

        let lib = node.get_library();
        let vhdl_lib = lib.as_vhdl_name().unwrap().clone();
        // println!("{} {}", source_file.get_file(), source_file.get_library());

        // add all entities to a graph and store architectures for later analysis
        symbols.into_iter().for_each(|f| {
            let name = f.as_name();
            match f {
                VerilogSymbol::Module(_) => {
                    component_pairs.insert(
                        LangIdentifier::Verilog(name.unwrap().clone()),
                        LangIdentifier::Vhdl(vhdl_lib.clone()),
                    );
                    // add primary design units into the graph
                    graph_map.add_node(
                        CompoundIdentifier::new(
                            lib.clone(),
                            LangIdentifier::Verilog(name.unwrap().clone()),
                        ),
                        HdlNode::new(HdlSymbol::Verilog(f), node),
                    );
                }
                VerilogSymbol::Config(_) => {
                    component_pairs.insert(
                        LangIdentifier::Verilog(name.unwrap().clone()),
                        LangIdentifier::Vhdl(vhdl_lib.clone()),
                    );
                    // add primary design units into the graph
                    graph_map.add_node(
                        CompoundIdentifier::new(
                            lib.clone(),
                            LangIdentifier::Verilog(name.unwrap().clone()),
                        ),
                        HdlNode::new(HdlSymbol::Verilog(f), node),
                    );
                }
                VerilogSymbol::Primitive(_) => {
                    component_pairs.insert(
                        LangIdentifier::Verilog(name.unwrap().clone()),
                        LangIdentifier::Vhdl(vhdl_lib.clone()),
                    );
                    // add primary design units into the graph
                    graph_map.add_node(
                        CompoundIdentifier::new(
                            lib.clone(),
                            LangIdentifier::Verilog(name.unwrap().clone()),
                        ),
                        HdlNode::new(HdlSymbol::Verilog(f), node),
                    );
                }
            }
        });
        Ok(())
    }

    pub fn create_systemverilog_node<'a, 'b>(
        graph_map: &'b mut GraphMap<CompoundIdentifier, HdlNode<'a>, ()>,
        node: &'a IpFileNode,
        component_pairs: &'b mut HashMap<LangIdentifier, LangIdentifier>,
    ) -> Result<(), Fault> {
        let contents = lang::read_to_string(&node.get_file())?;
        let symbols = match SystemVerilogParser::read(&contents) {
            Ok(s) => s.into_symbols(),
            Err(e) => Err(ParseError::SourceCodeError(
                node.get_file().clone(),
                e.to_string(),
            ))?,
        };

        let lib = node.get_library();
        let vhdl_lib = lib.as_vhdl_name().unwrap().clone();
        // println!("{} {}", source_file.get_file(), source_file.get_library());

        // add all entities to a graph and store architectures for later analysis
        symbols.into_iter().for_each(|f| {
            let name = f.as_name();
            match f {
                SystemVerilogSymbol::Module(_) => {
                    component_pairs.insert(
                        LangIdentifier::SystemVerilog(name.unwrap().clone()),
                        LangIdentifier::Vhdl(vhdl_lib.clone()),
                    );
                    // add primary design units into the graph
                    graph_map.add_node(
                        CompoundIdentifier::new(
                            lib.clone(),
                            LangIdentifier::SystemVerilog(name.unwrap().clone()),
                        ),
                        HdlNode::new(HdlSymbol::SystemVerilog(f), node),
                    );
                }
                SystemVerilogSymbol::Config(_)
                | SystemVerilogSymbol::Class(_)
                | SystemVerilogSymbol::Package(_)
                | SystemVerilogSymbol::Primitive(_)
                | SystemVerilogSymbol::Checker(_)
                | SystemVerilogSymbol::Program(_)
                | SystemVerilogSymbol::Interface(_) => {
                    component_pairs.insert(
                        LangIdentifier::Verilog(name.unwrap().clone()),
                        LangIdentifier::Vhdl(vhdl_lib.clone()),
                    );
                    // add primary design units into the graph
                    graph_map.add_node(
                        CompoundIdentifier::new(
                            lib.clone(),
                            LangIdentifier::Verilog(name.unwrap().clone()),
                        ),
                        HdlNode::new(HdlSymbol::SystemVerilog(f), node),
                    );
                }
            }
        });
        Ok(())
    }

    pub fn create_vhdl_node<'a, 'b>(
        graph_map: &'b mut GraphMap<CompoundIdentifier, HdlNode<'a>, ()>,
        node: &'a IpFileNode,
        component_pairs: &'b mut HashMap<LangIdentifier, LangIdentifier>,
        sub_nodes: &'b mut Vec<(LangIdentifier, SubUnitNode<'a>)>,
    ) -> Result<(), Fault> {
        let contents = lang::read_to_string(&node.get_file())?;
        let symbols = match VHDLParser::read(&contents) {
            Ok(s) => s.into_symbols(),
            Err(e) => Err(ParseError::SourceCodeError(
                node.get_file().clone(),
                e.to_string(),
            ))?,
        };

        let lib = node.get_library();
        let vhdl_lib = lib.as_vhdl_name().unwrap().clone();

        // add all entities to a graph and store architectures for later analysis
        let mut iter = symbols.into_iter().filter_map(|f| {
            match f {
                VhdlSymbol::Entity(_) => {
                    component_pairs.insert(
                        LangIdentifier::Vhdl(f.as_entity().unwrap().get_name().clone()),
                        LangIdentifier::Vhdl(vhdl_lib.clone()),
                    );
                    Some(f)
                }
                VhdlSymbol::Package(_) => Some(f),
                VhdlSymbol::Context(_) => Some(f),
                VhdlSymbol::Architecture(arch) => {
                    sub_nodes.push((
                        LangIdentifier::Vhdl(vhdl_lib.clone()),
                        SubUnitNode::new(SubUnit::from_arch(arch), node),
                    ));
                    None
                }
                VhdlSymbol::Configuration(cfg) => {
                    sub_nodes.push((
                        LangIdentifier::Vhdl(vhdl_lib.clone()),
                        SubUnitNode::new(SubUnit::from_config(cfg), node),
                    ));
                    None
                }
                // package bodies are usually in same design file as package
                VhdlSymbol::PackageBody(pb) => {
                    sub_nodes.push((
                        LangIdentifier::Vhdl(vhdl_lib.clone()),
                        SubUnitNode::new(SubUnit::from_body(pb), node),
                    ));
                    None
                }
            }
        });
        while let Some(e) = iter.next() {
            // add primary design units into the graph
            graph_map.add_node(
                CompoundIdentifier::new_vhdl(
                    lib.as_vhdl_name().unwrap().clone(),
                    e.get_name().unwrap().clone(),
                ),
                HdlNode::new(HdlSymbol::Vhdl(e), node),
            );
        }
        Ok(())
    }

    pub fn connect_edges_from_verilog<'b, 'a>(
        graph_map: &'b mut GraphMap<CompoundIdentifier, HdlNode<'a>, ()>,
        component_pairs: &'b mut HashMap<LangIdentifier, LangIdentifier>,
        only_components: bool,
    ) -> () {
        // filter for the verilog/systemverilog nodes for connections
        let mut module_nodes_iter = graph_map
            .get_map()
            .values()
            .filter_map(|f| {
                if f.as_ref().get_lang() == Lang::Verilog
                    || f.as_ref().get_lang() == Lang::SystemVerilog
                {
                    let sym = f.as_ref().get_symbol();
                    match only_components {
                        true => {
                            if sym.is_component() == true {
                                Some((
                                    f.as_ref().get_library(),
                                    sym.get_name(),
                                    sym.as_module().unwrap().get_edge_list_entities(),
                                    sym.as_module().unwrap().get_edge_list_entities(),
                                ))
                            } else {
                                None
                            }
                        }
                        false => {
                            if let Some(m) = sym.as_module() {
                                Some((
                                    f.as_ref().get_library(),
                                    sym.get_name(),
                                    m.get_edge_list_entities(),
                                    sym.get_refs()
                                        .unwrap_or(&HashSet::new())
                                        .into_iter()
                                        .map(|c| c.clone())
                                        .collect(),
                                ))
                            } else {
                                Some((
                                    f.as_ref().get_library(),
                                    sym.get_name(),
                                    Vec::new(),
                                    sym.get_refs()
                                        .unwrap_or(&HashSet::new())
                                        .into_iter()
                                        .map(|c| c.clone())
                                        .collect(),
                                ))
                            }
                        }
                    }
                } else {
                    None
                }
            })
            // collects as (library, name, entity edges, ref edges)
            .collect::<Vec<(
                LangIdentifier,
                LangIdentifier,
                Vec<CompoundIdentifier>,
                Vec<CompoundIdentifier>,
            )>>()
            .into_iter();

        // go through the filtered nodes and connect to other design elements/units that exist
        while let Some((lib, name, mods, deps)) = module_nodes_iter.next() {
            let node_name = CompoundIdentifier::new(lib, name);
            // create edges by ordered edge list (for entities)
            let mut deps = deps.into_iter();
            while let Some(dep) = &deps.next() {
                // need to locate the key with a suffix matching `dep` if it was a component instantiation
                if dep.get_prefix().is_none() {
                    if let Some(lib) = component_pairs.get(dep.get_suffix()) {
                        let b = graph_map.add_edge_by_key(
                            &CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone()),
                            &node_name,
                            (),
                        );
                        // create black box only if this dependency appeared as an module instantiation
                        if mods.contains(dep) {
                            match b {
                                // create black box entity
                                EdgeStatus::MissingSource => {
                                    let dep_name = CompoundIdentifier::new(
                                        lib.clone(),
                                        dep.get_suffix().clone(),
                                    );

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
                        }
                    // this entity does not exist or was not logged
                    } else {
                        if mods.contains(dep) {
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
                    }
                // the dependency has a prefix (a library) with it
                } else {
                    graph_map.add_edge_by_key(dep, &node_name, ());
                };
            }
        }
    }

    /// Builds a graph of design units. Used for planning
    pub fn build_full_graph<'a>(
        files: &'a Vec<IpFileNode>,
    ) -> Result<GraphMap<CompoundIdentifier, HdlNode<'a>, ()>, Fault> {
        let mut graph_map: GraphMap<CompoundIdentifier, HdlNode, ()> = GraphMap::new();

        let mut sub_nodes: Vec<(LangIdentifier, SubUnitNode)> = Vec::new();

        // store the (suffix, prefix) for all entities
        let mut component_pairs: HashMap<LangIdentifier, LangIdentifier> = HashMap::new();
        // read all files
        for source_file in files {
            // println!("{}", source_file.get_file());
            match source_file.get_language() {
                Lang::Vhdl => Self::create_vhdl_node(
                    &mut graph_map,
                    source_file,
                    &mut component_pairs,
                    &mut sub_nodes,
                )?,
                Lang::Verilog => {
                    Self::create_verilog_node(&mut graph_map, source_file, &mut component_pairs)?
                }
                Lang::SystemVerilog => Self::create_systemverilog_node(
                    &mut graph_map,
                    source_file,
                    &mut component_pairs,
                )?,
            }
        }

        // add connections for verilog and systemverilog
        Self::connect_edges_from_verilog(&mut graph_map, &mut component_pairs, false);

        // go through all architectures and make the connections
        let mut sub_nodes_iter = sub_nodes.into_iter();
        while let Some((lib, node)) = sub_nodes_iter.next() {
            let node_name = CompoundIdentifier::new(
                lib,
                LangIdentifier::Vhdl(node.get_sub().get_entity().clone()),
            );

            // link to the owner and add architecture's source file
            let entity_node = match graph_map.get_node_by_key_mut(&node_name) {
                Some(en) => en,
                // @todo: issue error because the entity (owner) is not declared
                None => continue,
            };
            entity_node.as_ref_mut().add_file(node.get_file());
            // create edges (this is very important)
            for dep in node.get_sub().get_edge_list() {
                // println!("{:?}", dep);
                // need to locate the key with a suffix matching `dep` if it was a component instantiation
                if dep.get_prefix().is_none() == true {
                    if let Some(lib) = component_pairs.get(dep.get_suffix()) {
                        graph_map.add_edge_by_key(
                            &CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone()),
                            &node_name,
                            (),
                        );
                    }
                } else {
                    graph_map.add_edge_by_key(dep, &node_name, ());
                };
            }
        }

        // go through all nodes and make the connections
        let idens: Vec<CompoundIdentifier> = graph_map
            .get_map()
            .into_iter()
            .map(|(k, _)| k.clone())
            .collect();
        for iden in idens {
            let references: Vec<CompoundIdentifier> = if let Some(refs) = graph_map
                .get_node_by_key(&iden)
                .unwrap()
                .as_ref()
                .get_symbol()
                .get_refs()
            {
                refs.into_iter().map(|rr| rr.clone()).collect()
            } else {
                // skip this unit
                continue;
            };

            // NOTE: This is a very important and fragile loop
            for dep in &references {
                let working = LangIdentifier::new_working();
                // re-route the library prefix to the current unit's library
                let dep_adjusted = CompoundIdentifier::new(
                    iden.get_prefix().unwrap_or(&working).clone(),
                    dep.get_suffix().clone(),
                );
                // if the dep is using "work", match it with the identifier's library
                let (dep_adjusted, _lib_was_work) = if let Some(lib) = dep.get_prefix() {
                    match lib == &working {
                        true => (&dep_adjusted, true),
                        false => (dep, false),
                    }
                } else {
                    (dep, false)
                };
                // println!("{} {} ... {}", iden, dep, dep_adjusted);
                // verify the dep exists
                let _stat = graph_map.add_edge_by_key(dep_adjusted, &iden, ());
                // println!("{:?} -> {:?} ... {:?}", dep_adjusted.to_string(), &iden.to_string(), _stat);

                // NOTE: This code section will allow a fallback in case of trying to explicitly use a library
                // called "work" when the actual working library is not explicitly called "work" for an external
                // reference. However, this is not intended behavior and the ip.library field should not allow a
                // user to explicitly set the value to "work" for VHDL's sake.
                // match stat {
                //     EdgeStatus::MissingSource => {
                //         // okay... maybe the actual library is called "work"
                //         if lib_was_work == true {
                //             let _next_stat = graph_map.add_edge_by_key(dep, &iden, ());
                //         }
                //     },
                //     _ => ()
                // }
            }
        }
        Ok(graph_map)
    }

    /// Writes the lockfile according to the constructed `ip_graph`. Only writes if the lockfile is
    /// out of date or `force` is `true`.
    pub fn write_lockfile<'c>(
        target: &Ip,
        ip_graph: &GraphMap<IpSpec, IpNode, ()>,
        force: bool,
        verbose: bool,
        catalog: &Catalog<'c>,
    ) -> Result<(), Fault> {
        // only modify the lockfile if it is out-of-date
        if target.can_use_lock(&catalog) == false || force == true {
            // create build list
            let build_list: Vec<&Ip> = ip_graph
                .get_map()
                .iter()
                .map(|p| p.1.as_ref().as_original_ip())
                .collect();
            let lock = LockFile::from_build_list(build_list, target)?;
            lock.save_to_disk(target.get_root())?;

            if target.get_lock() != &lock {
                if verbose == true {
                    crate::info!("lockfile updated");
                }
            } else {
                if verbose == true {
                    crate::info!("lockfile experienced no changes");
                }
            }
        } else {
            if verbose == true {
                crate::info!("lockfile experienced no changes");
            }
        }
        Ok(())
    }

    /// Maps the local index to the global index between two different maps.
    ///
    /// Assumes `local` is a subset of `global`.
    pub fn local_to_global<'a>(
        local_index: usize,
        global: &'a GraphMap<CompoundIdentifier, HdlNode, ()>,
        local: &GraphMap<&CompoundIdentifier, &HdlNode, &()>,
    ) -> &'a Node<HdlNode<'a>> {
        global
            .get_node_by_key(local.get_key_by_index(local_index).unwrap())
            .unwrap()
    }

    fn detect_bench(
        _graph: &GraphMap<CompoundIdentifier, HdlNode, ()>,
        local: &GraphMap<&CompoundIdentifier, &HdlNode, &()>,
        working_lib: &LangIdentifier,
        bench: &Option<Identifier>,
        top: &Option<Identifier>,
    ) -> Result<(Option<usize>, Option<usize>), PlanError> {
        Ok(if let Some(t) = &bench {
            match local.get_node_by_key(&&CompoundIdentifier::new_vhdl(
                working_lib.as_vhdl_name().unwrap().clone(),
                t.clone(),
            )) {
                // verify the unit is an entity that is a testbench
                Some(node) => {
                    if node.as_ref().get_symbol().is_component() == true {
                        if node.as_ref().get_symbol().is_testbench() == false {
                            return Err(PlanError::BadTestbench(t.clone(), Hint::WantsTop))?;
                        }
                        // return the id from the local graph
                        (None, Some(node.index()))
                    } else {
                        return Err(PlanError::BadEntity(t.clone()))?;
                    }
                }
                None => return Err(PlanError::UnknownEntity(t.clone()))?,
            }
        // try to find the naturally occurring top-level if user did not provide --bench and did not provide --top
        } else if top.is_none() {
            match local.find_root() {
                // only detected a single root
                Ok(n) => {
                    let n = local
                        .get_node_by_key(local.get_key_by_index(n.index()).unwrap())
                        .unwrap();
                    // verify the root is a testbench
                    if n.as_ref().get_symbol().is_component() == true {
                        if n.as_ref().get_symbol().is_testbench() == true {
                            (None, Some(n.index()))
                        // otherwise we found the toplevel node that is not a testbench "natural top"
                        } else {
                            // return the local index
                            (Some(n.index()), None)
                        }
                    } else {
                        (None, None)
                    }
                }
                Err(e) => match e.len() {
                    0 => (None, None),
                    _ => {
                        // filter to only testbenches
                        let tbs: Vec<&usize> = e
                            .iter()
                            .filter(|i| {
                                local
                                    .get_node_by_index(**i)
                                    .unwrap()
                                    .as_ref()
                                    .get_symbol()
                                    .is_testbench()
                                    == true
                            })
                            .collect();
                        match tbs.len() {
                            0 => (None, None),
                            1 => (None, Some(*tbs[0])),
                            _ => {
                                return Err(PlanError::Ambiguous(
                                    "testbenches".to_string(),
                                    tbs.into_iter()
                                        .map(|f| {
                                            local
                                                .get_map()
                                                .get(local.get_graph().get_node(*f).unwrap())
                                                .unwrap()
                                                .as_ref()
                                                .get_symbol()
                                                .get_name()
                                                .clone()
                                        })
                                        .collect(),
                                    Hint::BenchSpecify,
                                ))?
                            }
                        }
                    }
                },
            }
        } else {
            // still could possibly be found by top level if top is some
            (None, None)
        })
    }

    /// Given a `graph` and optionally a `bench`, detect the index corresponding
    /// to the top.
    ///
    /// This function looks and checks if there is a single predecessor to the
    /// `bench` node.
    fn detect_top(
        _graph: &GraphMap<CompoundIdentifier, HdlNode, ()>,
        local: &GraphMap<&CompoundIdentifier, &HdlNode, &()>,
        working_lib: &LangIdentifier,
        natural_top: Option<usize>,
        mut bench: Option<usize>,
        top: &Option<Identifier>,
        allow_bench: bool,
    ) -> Result<(Option<usize>, Option<usize>), PlanError> {
        // determine the top-level node index
        let top: Option<usize> = if let Some(t) = &top {
            match local.get_node_by_key(&&CompoundIdentifier::new_vhdl(
                working_lib.as_vhdl_name().unwrap().clone(),
                t.clone(),
            )) {
                Some(node) => {
                    // verify the unit is an entity that is not a testbench
                    if node.as_ref().get_symbol().is_component() == true {
                        if node.as_ref().get_symbol().is_testbench() == true {
                            if allow_bench == true {
                                return Err(PlanError::BadDut(t.clone(), Hint::BenchSpecify))?;
                            } else {
                                return Err(PlanError::BadTop(t.clone(), Hint::WantsTestbench))?;
                            }
                        }
                    } else {
                        // return Err(PlanError::BadEntity(t.clone()))?;
                    }
                    let n: usize = node.index();
                    // try to detect top level testbench
                    if bench.is_none() == true && allow_bench == true {
                        // check if only 1 is a testbench
                        let benches: Vec<usize> = local
                            .get_graph()
                            .successors(n)
                            .filter(|f| {
                                local
                                    .get_node_by_index(*f)
                                    .unwrap()
                                    .as_ref()
                                    .get_symbol()
                                    .is_testbench()
                            })
                            .collect();
                        // detect the testbench
                        bench = match benches.len() {
                            0 => None,
                            1 => Some(*benches.first().unwrap()),
                            _ => {
                                return Err(PlanError::Ambiguous(
                                    "testbenches".to_string(),
                                    benches
                                        .into_iter()
                                        .map(|f| {
                                            local
                                                .get_node_by_index(f)
                                                .unwrap()
                                                .as_ref()
                                                .get_symbol()
                                                .get_name()
                                        })
                                        .collect(),
                                    Hint::BenchSpecify,
                                ))?
                            }
                        };
                    }
                    // return the index from the local graph
                    Some(n)
                }
                None => return Err(PlanError::UnknownEntity(t.clone()))?,
            }
        } else {
            match natural_top {
                Some(nt) => Some(nt),
                None => {
                    if let Some(b) = bench {
                        let entities: Vec<(usize, &HdlSymbol)> = local
                            .get_graph()
                            .predecessors(b)
                            .filter_map(|f| {
                                if local
                                    .get_node_by_index(f)
                                    .unwrap()
                                    .as_ref()
                                    .get_symbol()
                                    .is_component()
                                {
                                    Some((
                                        f,
                                        local.get_node_by_index(f).unwrap().as_ref().get_symbol(),
                                    ))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        match entities.len() {
                            // catch this error when it occurs during plan to allow for tbs without entities
                            0 => {
                                return Err(PlanError::TestbenchNoTest(
                                    local.get_key_by_index(b).unwrap().get_suffix().clone(),
                                ))
                            }
                            1 => Some(entities[0].0),
                            _ => {
                                return Err(PlanError::Ambiguous(
                                    "components instantiated in the testbench".to_string(),
                                    entities
                                        .into_iter()
                                        .map(|f| {
                                            local
                                                .get_node_by_index(f.0)
                                                .unwrap()
                                                .as_ref()
                                                .get_symbol()
                                                .get_name()
                                        })
                                        .collect(),
                                    Hint::DutSpecify,
                                ))?
                            }
                        }
                    } else {
                        // auto-detect top-level if no testbench was given
                        let tops: Vec<(usize, &HdlSymbol)> = local
                            .get_map()
                            .iter()
                            .filter_map(|(_k, v)| {
                                if v.as_ref().get_symbol().is_component()
                                    && v.as_ref().get_symbol().is_testbench() == false
                                {
                                    Some((v.index(), v.as_ref().get_symbol()))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        // filter to get all potential candidates
                        let tops: Vec<(usize, &HdlSymbol)> = tops
                            .into_iter()
                            .filter(|(i, _v)| {
                                local
                                    .get_graph()
                                    .successors(*i)
                                    .filter(|k| {
                                        let s = local
                                            .get_node_by_index(*k)
                                            .unwrap()
                                            .as_ref()
                                            .get_symbol();

                                        s.is_testbench() == false && s.is_component()
                                    })
                                    .count()
                                    == 0
                            })
                            .collect();

                        match tops.len() {
                            // catch this error when it occurs during plan to allow for tbs without entities
                            0 => None,
                            1 => Some(tops[0].0),
                            _ => {
                                return Err(PlanError::Ambiguous(
                                    if allow_bench == false {
                                        format!("top-level design units")
                                    } else {
                                        format!("design-under-test units")
                                    },
                                    tops.into_iter()
                                        .map(|f| {
                                            local
                                                .get_node_by_index(f.0)
                                                .unwrap()
                                                .as_ref()
                                                .get_symbol()
                                                .get_name()
                                        })
                                        .collect(),
                                    if allow_bench == true {
                                        Hint::DutSpecify
                                    } else {
                                        Hint::TopSpecify
                                    },
                                ))?
                            }
                        }
                    }
                }
            }
        };
        Ok((top, bench))
    }

    /// Modifies the `list` to only have a list of unique elements while preserving their original
    /// order.
    ///
    /// Removes all duplicate elements found after the first occurence of said element.
    fn remove_multi_occurences<T: Eq + Hash>(list: &Vec<T>) -> Vec<&T> {
        let mut result = Vec::new();
        // be prepared to store no more than the amount of elements in `list`
        result.reserve(list.len());
        // gradually build a set to track duplicates
        let mut set = HashSet::<&T>::new();

        for elem in list {
            if set.insert(elem) == true {
                result.push(elem);
            }
        }
        result
    }

    /// This function transforms the list of indices from `min_order` in topologically-sorted order
    /// to the list of files in topologically-sorted order based on the information
    /// in the `global_graph`.
    ///
    /// Several files may be associated with an index in the `global_graph`, so it is important
    /// to account for those too.
    fn determine_file_order<'a>(
        global_graph: &'a GraphMap<CompoundIdentifier, HdlNode, ()>,
        min_order: Vec<usize>,
    ) -> Vec<&'a IpFileNode<'a>> {
        // gather the files from each node in-order (multiple files can exist for a node)
        let mut file_map = HashMap::<String, (&IpFileNode, Vec<&HdlNode>)>::new();
        let mut file_order = Vec::<String>::new();

        for i in &min_order {
            // access the node key and access the files associated with this key (the dependencies)
            let ipfs = global_graph
                .get_node_by_index(*i)
                .unwrap()
                .as_ref()
                .get_associated_files();
            // handle each associated file in the list
            ipfs.into_iter().for_each(|&ip_file_node| {
                // collect all dependencies in the graph from this node
                let mut preds: Vec<&HdlNode> = global_graph
                    .predecessors(*i)
                    .into_iter()
                    .map(|ip_file_node| ip_file_node.1)
                    .collect();
                // merge dependencies together from various primary design units
                match file_map.get_mut(ip_file_node.get_file()) {
                    // update the existing node by merging dependencies together
                    Some((_file_node, deps)) => {
                        deps.append(&mut preds);
                    }
                    // enter the new unmarked node and its dependencies
                    None => {
                        file_order.push(ip_file_node.get_file().clone());
                        file_map.insert(ip_file_node.get_file().clone(), (ip_file_node, preds));
                    }
                }
            });
        }

        // build a graph where nodes are files
        let mut file_graph: GraphMap<&'a IpFileNode<'a>, (), ()> = GraphMap::new();

        for file_name in &file_order {
            let (node, deps) = file_map.get(file_name).unwrap();
            // make sure the node exists in the graph before making edge connections
            if file_graph.has_node_by_key(&node) == false {
                file_graph.add_node(node, ());
            }
            for &ifn in deps {
                for pred_node in ifn.get_associated_files() {
                    // make sure the node exists before creating edges
                    if file_graph.has_node_by_key(&pred_node) == false {
                        file_graph.add_node(pred_node, ());
                    }
                    // add edge between them (this function prevents self-loops)
                    let _ = file_graph.add_edge_by_key(pred_node, node, ());
                }
            }
        }
        // topologically sort and transform into list of the file nodes
        file_graph
            .get_graph()
            .topological_sort()
            .into_iter()
            .map(|i| *file_graph.get_key_by_index(i).unwrap())
            .collect()
    }

    /// Filters out the local nodes existing within the current IP from the `global_graph`.
    pub fn compute_local_graph<'a>(
        global_graph: &'a GraphMap<CompoundIdentifier, HdlNode, ()>,
        target: &Ip,
    ) -> GraphMap<&'a CompoundIdentifier, &'a HdlNode<'a>, &'a ()> {
        let working_lib = target.get_hdl_library();
        // restrict graph to units only found within the current IP
        let local_graph: GraphMap<&CompoundIdentifier, &HdlNode, &()> = global_graph
            .iter()
            // traverse subset of graph by filtering only for working library entities (current lib)
            .filter(|f| match f.0.get_prefix() {
                Some(iden) => &LangIdentifier::from(iden.clone()) == &working_lib,
                None => false,
            })
            // filter by checking if the node's ip is the same as target
            .filter(|f| {
                let mut in_range: bool = true;
                for tag in f.1.get_associated_files() {
                    if tag.get_ip() != target {
                        in_range = false;
                        break;
                    }
                }
                in_range
            })
            .collect();

        local_graph
    }

    /// Writes the blueprint and env file to the build directory.
    fn create_outputs(
        blueprint: &Blueprint,
        mut envs: Environment,
        target_path: &PathBuf,
        top_name: &str,
        top_file: &str,
        top_json: &str,
        bench_name: &str,
        bench_file: &str,
        bench_json: &str,
        target: &Target,
        require_bench: bool,
    ) -> Result<PathBuf, Fault> {
        let output_path = target_path.join(target.get_name());
        // create a output build directorie(s) if they do not exist
        if output_path.exists() == false {
            fs::create_dir_all(&output_path).expect("could not create output directory");
        }

        // create a cache tag file if does not exist
        match Context::is_cache_tag_valid(target_path) {
            Ok(_) => (),
            Err(e) => fs::write(&e, context::CACHE_TAG)?,
        }

        // create the blueprint file
        let (blueprint_path, _) = blueprint.write(&output_path)?;

        // build upon existing environment variables to save in .env file
        envs = envs
            .add(EnvVar::with(
                environment::ORBIT_TOP_NAME,
                if require_bench == false {
                    &top_name
                } else {
                    ""
                },
            ))
            .add(EnvVar::with(
                environment::ORBIT_TOP_FILE,
                if require_bench == false {
                    &top_file
                } else {
                    ""
                },
            ))
            .add(EnvVar::with(
                environment::ORBIT_TOP_JSON,
                if require_bench == false {
                    &top_json
                } else {
                    ""
                },
            ))
            .add(EnvVar::with(
                environment::ORBIT_DUT_NAME,
                if require_bench == true { &top_name } else { "" },
            ))
            .add(EnvVar::with(
                environment::ORBIT_DUT_FILE,
                if require_bench == true { &top_file } else { "" },
            ))
            .add(EnvVar::with(
                environment::ORBIT_DUT_JSON,
                if require_bench == true { &top_json } else { "" },
            ))
            .add(EnvVar::with(environment::ORBIT_TB_NAME, &bench_name))
            .add(EnvVar::with(environment::ORBIT_TB_FILE, &bench_file))
            .add(EnvVar::with(environment::ORBIT_TB_JSON, &bench_json))
            .add(EnvVar::with(
                environment::ORBIT_BLUEPRINT,
                &blueprint.get_filename(),
            ))
            .add(EnvVar::with(environment::ORBIT_TARGET, target.get_name()));

        environment::save_environment(&envs, &output_path)?;

        Ok(blueprint_path)
    }
}

#[derive(Debug)]
pub enum PlanError {
    BadTestbench(Identifier, Hint),
    BadTop(Identifier, Hint),
    BadDut(Identifier, Hint),
    BadEntity(Identifier),
    TestbenchNoTest(LangIdentifier), // this error gets skipped
    UnknownUnit(Identifier),
    UnknownEntity(Identifier),
    Ambiguous(String, Vec<LangIdentifier>, Hint),
    Empty,
}

impl std::error::Error for PlanError {}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TestbenchNoTest(id) => {
                write!(f, "zero entities are tested in testbench \"{}\"", id)
            }
            Self::UnknownEntity(id) => write!(
                f,
                "local ip does not contain any component named \"{}\"",
                id
            ),
            Self::Empty => write!(f, "zero components found in the local ip"),
            Self::BadEntity(id) => write!(f, "design element \"{}\" is not a component", id),
            Self::BadTestbench(id, hint) => {
                write!(f, "component \"{}\" is not a testbench{}", id, hint,)
            }
            Self::BadTop(id, hint) => write!(
                f,
                "component \"{}\" is a testbench and cannot be top{}",
                id, hint
            ),
            Self::BadDut(id, hint) => write!(
                f,
                "component \"{}\" is a testbench and cannot be dut{}",
                id, hint
            ),
            Self::UnknownUnit(id) => {
                write!(
                    f,
                    "no primary design unit named \"{}\" in the current ip",
                    id
                )
            }
            Self::Ambiguous(name, tbs, hint) => write!(
                f,
                "multiple {} were found:\n{}{}",
                name,
                tbs.iter().enumerate().fold(String::new(), |sum, (i, x)| {
                    sum + &format!("    {}{}", x, if i + 1 < tbs.len() { "\n" } else { "" })
                }),
                hint,
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn remove_multi_occur() {
        // removes
        let arr = vec![1, 2, 2, 3, 4];
        assert_eq!(Plan::remove_multi_occurences(&arr), vec![&1, &2, &3, &4]);

        let arr = vec![1, 2, 2, 3, 4, 2, 1];
        assert_eq!(Plan::remove_multi_occurences(&arr), vec![&1, &2, &3, &4]);

        let arr = vec![1, 1, 1, 1, 1];
        assert_eq!(Plan::remove_multi_occurences(&arr), vec![&1]);

        // no changes
        let arr = vec![9, 8, 7, 6, 5, 4];
        assert_eq!(
            Plan::remove_multi_occurences(&arr),
            vec![&9, &8, &7, &6, &5, &4]
        );
    }
}

// impl Plan {
//     // DEPRECATED: This function may be outdated- was used when `plan` used to be a
//     // dedicated subcommand.

//     fn execute(self, c: &Context) -> Result<(), Fault> {
//         // locate the target provided from the command-line
//         let target = c.select_target(&self.target, self.list == false, true)?;

//         // display targets list and exit
//         if self.list == true {
//             match target {
//                 // display entire contents about the particular plugin
//                 Some(tg) => println!("{}", tg),
//                 // display quick overview of all plugins
//                 None => print!(
//                     "{}",
//                     Target::list_targets(
//                         &mut c
//                             .get_config()
//                             .get_targets()
//                             .values()
//                             .into_iter()
//                             .collect::<Vec<&&Target>>(),
//                         None,
//                     )
//                 ),
//             }
//             return Ok(());
//         }

//         // unwrap because at this point the target must exist
//         let target = target.unwrap();

//         // check that user is in an IP directory
//         c.jump_to_working_ip()?;

//         // store the working ip struct
//         let working_ip = Ip::load(c.get_ip_path().unwrap().clone(), true, false)?;

//         // assemble the catalog
//         let mut catalog = Catalog::new()
//             .installations(c.get_cache_path())?
//             .downloads(c.get_downloads_path())?;

//         // @todo: recreate the ip graph from the lockfile, then read each installation
//         // see Install::install_from_lock_file

//         // this code is only ran if the lock file matches the manifest and we aren't force to recompute
//         if working_ip.can_use_lock(&catalog) == true && self.force == false {
//             let le: LockEntry = LockEntry::from((&working_ip, true));
//             let lf = working_ip.get_lock();

//             let env = Environment::new()
//                 // read config.toml for setting any env variables
//                 .from_config(c.get_config())?;
//             let vtable = StrSwapTable::new().load_environment(&env)?;

//             download_missing_deps(vtable, &lf, &le, &catalog, &c.get_config().get_protocols())?;
//             // recollect the downloaded items to update the catalog for installations
//             catalog = catalog.downloads(c.get_downloads_path())?;

//             install_missing_deps(&lf, &le, &catalog)?;
//             // recollect the installations to update the catalog for dependency graphing
//             catalog = catalog.installations(c.get_cache_path())?;
//         }

//         // determine the build directory (command-line arg overrides configuration setting)
//         let default_target_dir = c.get_target_dir();
//         let target_dir = match &self.target_dir {
//             Some(t_dir) => t_dir,
//             None => &default_target_dir,
//         };

//         let _ = Self::run(
//             &working_ip,
//             target_dir,
//             target,
//             catalog,
//             self.clean,
//             self.force,
//             self.only_lock,
//             self.all,
//             &self.bench,
//             &self.top,
//             &self.filesets,
//             &Scheme::default(),
//             false,
//             true,
//         );
//         Ok(())
//     }
// }
