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

use crate::commands::helps::get;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::ip::IpSpec;
use crate::core::ip::PartialIpSpec;
use crate::core::lang::sv::format::SystemVerilogFormat;
use crate::core::lang::verilog::symbols::module::Module;
use crate::core::lang::vhdl::format::VhdlFormat;
use crate::core::lang::vhdl::interface;
use crate::core::lang::vhdl::interface::Architectures;
use crate::core::lang::vhdl::symbols::entity::Entity;
use crate::core::lang::vhdl::token::Identifier as VhdlIdentifier;
use crate::core::lang::Lang;
use crate::core::lang::LangIdentifier;
use crate::core::lang::LangUnit;
use crate::error::Error;
use crate::error::Hint;
use crate::util::anyerror::{AnyError, Fault};
use colored::Colorize;
use std::env;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Get {
    unit: VhdlIdentifier,
    ip: Option<PartialIpSpec>,
    signals: bool,
    component: bool,
    instance: bool,
    library: bool,
    architectures: bool,
    json: bool,
    signal_prefix: String,
    signal_suffix: String,
    // NOTE: not done yet... requires more work with detecting generics in the datatype of the signals
    // const_prefix: String,
    // const_suffix: String,
    name: Option<VhdlIdentifier>,
}

impl Subcommand<Context> for Get {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(get::HELP))?;
        Ok(Self {
            signals: cli.check(Arg::flag("signals").switch('s'))?,
            component: cli.check(Arg::flag("component").switch('c'))?,
            instance: cli.check(Arg::flag("instance").switch('i'))?,
            library: cli.check(Arg::flag("library").switch('l'))?,
            architectures: cli.check(Arg::flag("architecture").switch('a'))?,
            json: cli.check(Arg::flag("json"))?,
            signal_prefix: cli
                .get(Arg::option("signal-prefix").value("str"))?
                .unwrap_or_default(),
            signal_suffix: cli
                .get(Arg::option("signal-suffix").value("str"))?
                .unwrap_or_default(),
            // const_prefix: cli
            //     .get(Arg::option("const-prefix").value("value"))?
            //     .unwrap_or_default(),
            // const_suffix: cli
            //     .get(Arg::option("const-suffix").value("value"))?
            //     .unwrap_or_default(),
            ip: cli.get(Arg::option("ip").value("spec"))?,
            name: cli.get(Arg::option("name").value("identifier"))?,
            unit: cli.require(Arg::positional("unit"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // --name can only be used with --instance is set
        if self.name.is_some() && self.instance == false {
            return Err(AnyError(format!(
                "'{}' can only be used with '{}'",
                "--name".yellow(),
                "--instance".yellow()
            )))?;
        }

        // @todo: load the catalog
        let catalog = Catalog::new()
            // .store(c.get_store_path())
            // .development(c.get_development_path().unwrap())?
            .installations(c.get_cache_path())?;

        let mut is_local_ip = false;
        // try to auto-determine the ip (check if in a working ip)
        let ip_path = if let Some(spec) = &self.ip {
            // @todo: find the path to the provided ip by searching through the catalog
            if let Some(lvl) = catalog.translate_name(&spec.to_pkg_name())? {
                if let Some(slot) = lvl.get_install(spec.get_version()) {
                    slot.get_root().clone()
                } else {
                    return Err(AnyError(format!("ip {} does not exist in the cache", spec)))?;
                }
            } else {
                return Err(AnyError(format!("no ip found in cache")))?;
            }
        } else {
            let ip = Context::find_ip_path(&env::current_dir().unwrap());
            is_local_ip = true;
            if ip.is_none() == true {
                return Err(AnyError(format!("no ip provided or detected")))?;
            } else {
                ip.unwrap()
            }
        };

        // load the manifest from the path
        let ip = Ip::load(ip_path, is_local_ip)?;

        self.run(&ip, is_local_ip, &c)
    }
}

impl Get {
    fn run(&self, ip: &Ip, is_local: bool, c: &Context) -> Result<(), Fault> {
        // collect all hdl files and parse them
        let selected_unit = Self::fetch_entity(&ip, &LangIdentifier::Vhdl(self.unit.clone()))?;
        let unit = match selected_unit {
            Some(lu) => {
                // verify the unit is only set to public visibility when outside of ip
                if is_local == false && lu.get_visibility().is_public() == false {
                    return Err(Error::UnitIsWrongVisibility(
                        String::from("get"),
                        lu.get_name(),
                        lu.get_visibility().clone(),
                        Hint::ShowAvailableUnitsExternal(ip.get_man().get_ip().into_ip_spec()),
                    ))?;
                }
                // check to make sure it is a component
                if lu.is_component() {
                    lu
                } else {
                    let hint = match is_local {
                        true => Hint::ShowAvailableUnitsLocal,
                        false => {
                            Hint::ShowAvailableUnitsExternal(ip.get_man().get_ip().into_ip_spec())
                        }
                    };
                    return Err(Error::GetUnitNotComponent(lu.get_name().to_string(), hint))?;
                }
            }
            None => {
                let hint = match is_local {
                    true => Hint::ShowAvailableUnitsLocal,
                    false => Hint::ShowAvailableUnitsExternal(ip.get_man().get_ip().into_ip_spec()),
                };
                return Err(Error::GetUnitNotFound(self.unit.to_string(), hint))?;
            }
        };
        // determine how to handle unit display
        match unit.get_lang() {
            Lang::Vhdl => self.display_vhdl_entity(
                &ip,
                unit.get_vhdl_symbol().unwrap().as_entity().unwrap(),
                is_local,
                &c.get_vhdl_format(),
            ),
            Lang::Verilog => self.display_verilog_module(
                &ip,
                unit.get_verilog_symbol().unwrap().as_module().unwrap(),
                &c.get_sv_format(),
            ),
            Lang::SystemVerilog => self.display_verilog_module(
                &ip,
                unit.get_systemverilog_symbol()
                    .unwrap()
                    .as_module()
                    .unwrap(),
                &c.get_sv_format(),
            ),
        }?;

        Ok(())
    }

    fn display_vhdl_entity(
        &self,
        ip: &Ip,
        entity: &Entity,
        is_local: bool,
        fmt: &VhdlFormat,
    ) -> Result<(), Fault> {
        // determine if default print should appear
        let default_output = self.architectures == false
            && self.instance == false
            && self.json == false
            && self.signals == false
            && self.component == false
            && self.library == false;

        // make the library reference the current worki ng ip 'work' if its internal
        let lib = match is_local {
            true => VhdlIdentifier::new_working(),
            false => ip
                .get_man()
                .get_hdl_library()
                .as_vhdl_name()
                .unwrap()
                .clone(),
        };

        // display architectures
        if self.architectures == true {
            println!("{}", entity.get_architectures());
        }

        if fmt.is_syntax_highlighted() == false {
            // force turn off coloring output
            colored::control::set_override(false);
        }

        // display library declaration line if displaying instance
        if self.library == true {
            println!("{}", interface::library_statement(&lib));
        }

        // display component declaration
        if self.component == true || default_output == true {
            println!("{}", entity.into_component(&fmt));
        }

        // display signal declarations
        if self.signals == true {
            let constants = entity.into_constants(&fmt, "", "");
            if constants.is_empty() == false {
                println!("{}", constants);
            }
            let signals = entity.into_signals(&fmt, &self.signal_prefix, &self.signal_suffix);
            if signals.is_empty() == false {
                println!("{}", signals);
            }
        }

        // only display the direct entity instantiation code if not providing component code
        let lib = if self.component == true {
            None
        } else {
            Some(lib)
        };

        // display instantiation code
        if self.instance == true {
            println!(
                "{}",
                entity.into_instance(
                    &self.name,
                    &lib,
                    &fmt,
                    &self.signal_prefix,
                    &self.signal_suffix,
                    "",
                    "",
                )
            );
        }

        // print as json data
        if self.json == true {
            println!("{}", serde_json::to_string(&entity)?);
        }
        Ok(())
    }

    fn display_verilog_module(
        &self,
        _ip: &Ip,
        module: &Module,
        fmt: &SystemVerilogFormat,
    ) -> Result<(), Fault> {
        // determine if default print should appear
        let default_output = self.architectures == false
            && self.instance == false
            && self.json == false
            && self.signals == false
            && self.component == false
            && self.library == false;

        // display architectures
        if self.architectures == true {
            println!("{}N/A\n", Architectures::new(&Vec::new()));
        }

        if self.component == true || default_output == true {
            println!("{}\n", module.into_declaration(&fmt));
        }

        if self.signals == true {
            println!(
                "{}",
                module.into_wires(&self.signal_prefix, &self.signal_suffix, &fmt)
            );
        }

        if self.instance == true {
            println!(
                "{}",
                module.into_instance(&self.name, &self.signal_prefix, &self.signal_suffix, &fmt)
            );
        }

        // print as json data
        if self.json == true {
            println!("{}", serde_json::to_string(&module)?);
        }

        Ok(())
    }

    fn fetch_entity(ip: &Ip, name: &LangIdentifier) -> Result<Option<LangUnit>, Fault> {
        let mut files = ip.collect_units(true, false)?;
        let result = files.remove(name);
        Ok(result)
    }
}

#[derive(Debug)]
pub enum GetError {
    UnitNotFound(LangIdentifier, IpSpec),
    SuggestShow(String, Hint),
}

impl std::error::Error for GetError {}

impl std::fmt::Display for GetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnitNotFound(ent, spec) => {
                write!(f, "failed to find unit \"{}\" in ip \"{}\"", ent, spec)
            }
            Self::SuggestShow(err, hint) => {
                write!(f, "{}{}", err, hint)
            }
        }
    }
}
