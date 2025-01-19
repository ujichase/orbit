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

use std::io::BufReader;
use std::io::Read as ReadTrait;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use super::get::GetError;
use crate::commands::helps::read;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::ip::PartialIpSpec;
use crate::core::lang::lexer::Position;
use crate::core::lang::lexer::Token;
use crate::core::lang::sv::token::token::SystemVerilogToken;
use crate::core::lang::sv::token::tokenizer::SystemVerilogTokenizer;
use crate::core::lang::verilog::token::token::VerilogToken;
use crate::core::lang::verilog::token::tokenizer::VerilogTokenizer;
use crate::core::lang::vhdl::token::VhdlToken;
use crate::core::lang::vhdl::token::VhdlTokenizer;
use crate::core::lang::Lang;
use crate::core::lang::LangIdentifier;
use crate::error::Error;
use crate::error::Hint;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::sha256;
use std::fs;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

const SYS_DIR: &str = ".system";
const TMP_DIR: &str = "tmp";

#[derive(Debug, PartialEq)]
pub struct Read {
    unit: LangIdentifier,
    ip: Option<PartialIpSpec>,
    locate: bool,
    save: bool,
    no_clean: bool,
    start: Option<String>,
    end: Option<String>,
    comment: Option<String>,
    limit: Option<usize>,
}

impl Subcommand<Context> for Read {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(read::HELP))?;
        Ok(Read {
            // flags
            save: cli.check(Arg::flag("save"))?,
            locate: cli.check(Arg::flag("locate"))?,
            no_clean: cli.check(Arg::flag("no-clean"))?,
            // options
            limit: cli.get(Arg::option("limit").value("n"))?,
            ip: cli.get(Arg::option("ip").value("spec"))?,
            start: cli.get(Arg::option("start").value("code"))?,
            end: cli.get(Arg::option("end").value("code"))?,
            comment: cli.get(Arg::option("doc").value("code"))?,
            // positionals
            unit: cli.require(Arg::positional("unit"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // verify location is only set iff the file mode is enabled
        if self.locate == true && self.save == false {
            Err(AnyError(format!(
                "the flag '--locate' can only be set when using '--save'"
            )))?
        }

        // determine the destination
        let dest: PathBuf = c.get_home_path().join(SYS_DIR).join(TMP_DIR);

        // attempt to clean the tmp directory when --keep is disabled
        if dest.exists() == true && self.no_clean == false {
            // do not error if this procedure fails
            match std::fs::remove_dir_all(&dest) {
                Ok(_) => (),
                // @todo: warn user that directory was unable to be cleaned
                Err(_) => (),
            }
        }
        // cast the destination folder into an option based on if the user wants a file
        let dest = match self.save {
            true => Some(dest),
            false => None,
        };

        // checking external IP
        if let Some(spec) = &self.ip {
            // gather the catalog (all manifests)
            let catalog = Catalog::new().installations(c.get_cache_path())?;

            // access the requested ip
            match catalog.translate_name(&spec.to_pkg_name())? {
                Some(lvl) => {
                    let inst = match lvl.get_install(spec.get_version()) {
                        Some(i) => i,
                        None => panic!("version does not exist for this ip"),
                    };
                    self.run(inst, dest.as_ref(), false)
                }
                None => {
                    // the ip does not exist
                    return Err(Box::new(Error::IpNotFoundAnywhere(
                        spec.to_string(),
                        Hint::CatalogList,
                    )))?;
                }
            }
        // must be in an IP if omitting the pkgid
        } else {
            let ip = match c.get_ip_path() {
                Some(p) => Ip::load(p.to_path_buf(), true)?,
                None => return Err(AnyError(format!("not within an existing ip")))?,
            };

            self.run(&ip, dest.as_ref(), true)
        }
    }
}

impl Read {
    fn run(&self, target: &Ip, dest: Option<&PathBuf>, is_local: bool) -> Result<(), Fault> {
        let (path, loc, lang) = Self::read(&self.unit, &target, dest, is_local)?;

        // dump the file contents of the source code to the console if there was no destination
        let print_to_console = dest.is_none();

        // access the string contents
        let contents = fs::read_to_string(&path)?;

        let (start, end) = match lang {
            Lang::Vhdl => self.read_vhdl(&contents),
            Lang::Verilog => self.read_verilog(&contents),
            Lang::SystemVerilog => self.read_systemverilog(&contents),
        }?;

        let segment: String = {
            let iter = contents.split_terminator('\n');

            let iter = match &start {
                Some(p) => iter.skip(p.line() - 1),
                None => iter.skip(0),
            };
            let iter = match &end {
                Some(p) => {
                    iter.take(p.line() - start.as_ref().unwrap_or(&Position::new()).line() + 1)
                }
                None => iter.take(usize::MAX),
            };
            let iter = iter.map(|line| line.to_string() + "\n");

            match self.limit {
                Some(l) => iter.take(l).collect(),
                None => iter.collect(),
            }
        };

        println!(
            "{}",
            match print_to_console {
                // display the contents
                true => {
                    segment
                }
                // overwrite contents and display the file path
                false => {
                    let cut_code = (start.is_some() || end.is_some()) && self.locate == false;

                    let file = match cut_code {
                        true => {
                            // create and write a temporary file
                            let mut file = std::fs::OpenOptions::new()
                                .write(true)
                                .truncate(true)
                                .open(&path)?;

                            file.write(&segment.as_bytes())?;
                            file.flush()?;
                            file
                        }
                        false => std::fs::OpenOptions::new().read(true).open(&path)?,
                    };

                    // set to read-only
                    let mut perms = file.metadata()?.permissions();
                    perms.set_readonly(true);
                    file.set_permissions(perms)?;

                    // tack on the location to the file name
                    let path = {
                        if self.locate == true {
                            // update the location to point to
                            let loc = match start {
                                Some(s) => s,
                                None => match end {
                                    Some(e) => e,
                                    None => loc,
                                },
                            };
                            // append the location to the filepath
                            PathBuf::from({
                                let mut p = path.as_os_str().to_os_string();
                                p.push(&loc.to_string());
                                p
                            })
                        } else {
                            path
                        }
                    };

                    path.display().to_string()
                }
            }
        );
        Ok(())
    }

    /// Finds the filepath and file position for the provided primary design unit `unit`
    /// under the project `ip`.
    ///
    /// If `dest` contains a value, it will create a new directory at `dest` and copy
    /// the file to be read-only. If it is set to `None`, then it will open the
    /// file it is referencing (no copy).
    fn read(
        unit: &LangIdentifier,
        ip: &Ip,
        dest: Option<&PathBuf>,
        is_local: bool,
    ) -> Result<(PathBuf, Position, Lang), Fault> {
        // find the unit
        let units = ip.collect_units(true, false)?;

        // get the file data for the primary design unit
        let (source, position) = match units.get_key_value(unit) {
            Some((name, unit)) => {
                // verify the unit is not private when trying to read from nonlocal ip
                if unit.get_visibility().is_private() == true && is_local == false {
                    let spec = ip.get_man().get_ip().into_ip_spec();
                    return Err(Error::UnitIsWrongVisibility(
                        String::from("read"),
                        name.clone(),
                        unit.get_visibility().clone(),
                        Hint::ShowAvailableUnitsExternal(spec),
                    ))?;
                } else {
                    (
                        unit.get_source_file(),
                        match unit.get_lang() {
                            Lang::Vhdl => unit.get_vhdl_symbol().unwrap().get_position().clone(),
                            Lang::Verilog => {
                                unit.get_verilog_symbol().unwrap().get_position().clone()
                            }
                            Lang::SystemVerilog => unit
                                .get_systemverilog_symbol()
                                .unwrap()
                                .get_position()
                                .clone(),
                        },
                    )
                }
            }
            None => {
                let spec = ip.get_man().get_ip().into_ip_spec();
                return Err(GetError::SuggestShow(
                    GetError::UnitNotFound(unit.clone(), spec.clone()).to_string(),
                    Hint::ShowAvailableUnitsExternal(spec),
                ))?;
            }
        };

        let (checksum, bytes) = {
            // open the file and create a checksum
            let mut bytes = Vec::new();
            let file = std::fs::File::open(source)?;
            let mut reader = BufReader::new(file);
            reader.read_to_end(&mut bytes)?;
            (sha256::compute_sha256(&bytes), bytes)
        };

        let src = PathBuf::from(source);

        match dest {
            // return direct reference if no `dest` (within current ip)
            None => return Ok((src, position, units.get(unit).unwrap().get_lang())),
            Some(dest) => {
                // create new file under checksum directory
                let dest = dest.join(&checksum.to_string().get(0..10).unwrap());
                std::fs::create_dir_all(&dest)?;

                // add filename to destination path
                let dest = dest.join(src.file_name().unwrap());

                // try to remove file if it exists
                if dest.exists() == false || std::fs::remove_file(&dest).is_ok() {
                    // create and write a temporary file
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(&dest)?;
                    file.write(&bytes)?;
                    file.flush()?;
                }
                Ok((dest, position, units.get(unit).unwrap().get_lang()))
            }
        }
    }
}

// VHDL support
impl Read {
    fn read_vhdl(&self, contents: &str) -> Result<(Option<Position>, Option<Position>), Fault> {
        let start: Option<VhdlTokenizer> = match &self.start {
            Some(s) => Some(VhdlTokenizer::from_str(s)?),
            None => None,
        };
        let end: Option<VhdlTokenizer> = match &self.end {
            Some(s) => Some(VhdlTokenizer::from_str(s)?),
            None => None,
        };
        let comment: Option<VhdlTokenizer> = match &self.comment {
            Some(s) => Some(VhdlTokenizer::from_str(s)?),
            None => None,
        };

        let src_tokens = VhdlTokenizer::from_source_code(&contents).into_tokens_all();

        // perform a search on tokens
        let (start, end) = {
            // get the tokens
            let start_tokens = match &start {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VhdlToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let end_tokens = match &end {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VhdlToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let comment_tokens = match &comment {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VhdlToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            // search over the source code tokens
            let start = Self::find_location_vhdl(&src_tokens, &start_tokens);
            // limit based on starting index
            let remaining_tokens = match &start {
                Some(pos) => src_tokens
                    .into_iter()
                    .skip_while(|p| p.locate() <= pos)
                    .collect(),
                None => {
                    if self.start.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'start' code chunk"
                        )))?;
                    }
                    src_tokens
                }
            };

            let end = Self::find_location_vhdl(&remaining_tokens, &end_tokens);
            let remaining_tokens = match &end {
                Some(pos) => remaining_tokens
                    .into_iter()
                    .take_while(|p| p.locate() < pos)
                    .collect(),
                None => {
                    if self.end.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'end' code chunk"
                        )))?;
                    }
                    remaining_tokens
                }
            };

            // find the comment
            let first_token = Self::find_location_vhdl(&remaining_tokens, &comment_tokens);
            // grab all continuous '--' tokens immediately above/before this location
            let comment = match &first_token {
                Some(pos) => {
                    let mut line = pos.line();
                    // println!("{:?}", pos);
                    match remaining_tokens
                        .into_iter()
                        .rev()
                        .skip_while(|p| p.locate() >= pos)
                        .take_while(|p| {
                            // only take immediate comments grouped together
                            line -= 1;
                            p.as_type().as_comment().is_some() && p.locate().line() == line
                        })
                        .last()
                    {
                        Some(token) => Some(token.locate().clone()),
                        None => {
                            return Err(AnyError(format!(
                                "Zero comments associated with code chunk"
                            )))?
                        }
                    }
                }
                None => {
                    if self.comment.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'doc' code chunk"
                        )))?;
                    }
                    None
                }
            };

            let end = match &comment {
                Some(_) => first_token,
                None => end,
            };

            let start = match comment {
                Some(c) => Some(c),
                None => start,
            };

            (start, end)
        };
        Ok((start, end))
    }

    fn check_tokens_eq_vhdl(source: &VhdlToken, sub: &VhdlToken) -> bool {
        match sub {
            // skip EOF token
            &VhdlToken::EOF => true,
            // only match on the fact that they are comments
            VhdlToken::Comment(_) => source.as_comment().is_some(),
            _ => source == sub,
        }
    }

    fn find_location_vhdl(
        src_tokens: &Vec<Token<VhdlToken>>,
        find_tokens: &Vec<&Token<VhdlToken>>,
    ) -> Option<Position> {
        let mut tracking: bool;
        let mut src_tokens_iter = src_tokens.iter();
        while let Some(t) = src_tokens_iter.next() {
            // begin to see if we start tracking
            // println!("{:?} {:?}", find_tokens.first().unwrap().as_type(), t.as_type());

            if find_tokens.len() > 0
                && Self::check_tokens_eq_vhdl(t.as_type(), find_tokens.first().unwrap().as_type())
                    == true
            {
                // println!("{}", "HERE");
                let mut find_tokens_iter = find_tokens.iter().skip(1);
                tracking = true;
                while let Some(find_t) = find_tokens_iter.next() {
                    // skip the EOF token
                    if find_t.as_type() == &VhdlToken::EOF {
                        continue;
                    }
                    if let Some(source_t) = src_tokens_iter.next() {
                        // lost sight
                        if Self::check_tokens_eq_vhdl(source_t.as_type(), find_t.as_type()) == false
                        {
                            tracking = false;
                            break;
                        }
                    } else {
                        tracking = false;
                        break;
                    }
                }
                // initiate lock
                if tracking == true {
                    // @todo: return last index of list for skipping purposes
                    return Some(t.locate().clone());
                }
            }
            // @todo: handle follow better (keep iterating until hitting a wrong one)
        }
        None
    }
}

// Verilog support
impl Read {
    fn read_verilog(&self, contents: &str) -> Result<(Option<Position>, Option<Position>), Fault> {
        let start: Option<VerilogTokenizer> = match &self.start {
            Some(s) => Some(VerilogTokenizer::from_str(s)?),
            None => None,
        };
        let end: Option<VerilogTokenizer> = match &self.end {
            Some(s) => Some(VerilogTokenizer::from_str(s)?),
            None => None,
        };
        let comment: Option<VerilogTokenizer> = match &self.comment {
            Some(s) => Some(VerilogTokenizer::from_str(s)?),
            None => None,
        };

        let src_tokens = VerilogTokenizer::from_source_code(&contents).into_tokens_all();

        // perform a search on tokens
        let (start, end) = {
            // get the tokens
            let start_tokens = match &start {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VerilogToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let end_tokens = match &end {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VerilogToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let comment_tokens = match &comment {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VerilogToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            // search over the source code tokens
            let start = Self::find_location_verilog(&src_tokens, &start_tokens);
            // limit based on starting index
            let remaining_tokens = match &start {
                Some(pos) => src_tokens
                    .into_iter()
                    .skip_while(|p| p.locate() <= pos)
                    .collect(),
                None => {
                    if self.start.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'start' code chunk"
                        )))?;
                    }
                    src_tokens
                }
            };

            let end = Self::find_location_verilog(&remaining_tokens, &end_tokens);
            let remaining_tokens = match &end {
                Some(pos) => remaining_tokens
                    .into_iter()
                    .take_while(|p| p.locate() < pos)
                    .collect(),
                None => {
                    if self.end.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'end' code chunk"
                        )))?;
                    }
                    remaining_tokens
                }
            };

            // find the comment
            let first_token = Self::find_location_verilog(&remaining_tokens, &comment_tokens);
            // grab all continuous '--' tokens immediately above/before this location
            let comment = match &first_token {
                Some(pos) => {
                    let mut line = pos.line();
                    // println!("{:?}", pos);
                    match remaining_tokens
                        .into_iter()
                        .rev()
                        .skip_while(|p| p.locate() >= pos)
                        .take_while(|p| {
                            // only take immediate comments grouped together
                            line -= 1;
                            p.as_type().as_comment().is_some() && p.locate().line() == line
                        })
                        .last()
                    {
                        Some(token) => Some(token.locate().clone()),
                        None => {
                            return Err(AnyError(format!(
                                "Zero comments associated with code chunk"
                            )))?
                        }
                    }
                }
                None => {
                    if self.comment.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'doc' code chunk"
                        )))?;
                    }
                    None
                }
            };

            let end = match &comment {
                Some(_) => first_token,
                None => end,
            };

            let start = match comment {
                Some(c) => Some(c),
                None => start,
            };

            (start, end)
        };
        Ok((start, end))
    }

    fn check_tokens_eq_verilog(source: &VerilogToken, sub: &VerilogToken) -> bool {
        match sub {
            // skip EOF token
            &VerilogToken::EOF => true,
            // only match on the fact that they are comments
            VerilogToken::Comment(_) => source.as_comment().is_some(),
            _ => source == sub,
        }
    }

    fn find_location_verilog(
        src_tokens: &Vec<Token<VerilogToken>>,
        find_tokens: &Vec<&Token<VerilogToken>>,
    ) -> Option<Position> {
        let mut tracking: bool;
        let mut src_tokens_iter = src_tokens.iter();
        while let Some(t) = src_tokens_iter.next() {
            // begin to see if we start tracking
            // println!("{:?} {:?}", find_tokens.first().unwrap().as_type(), t.as_type());

            if find_tokens.len() > 0
                && Self::check_tokens_eq_verilog(
                    t.as_type(),
                    find_tokens.first().unwrap().as_type(),
                ) == true
            {
                // println!("{}", "HERE");
                let mut find_tokens_iter = find_tokens.iter().skip(1);
                tracking = true;
                while let Some(find_t) = find_tokens_iter.next() {
                    // skip the EOF token
                    if find_t.as_type() == &VerilogToken::EOF {
                        continue;
                    }
                    if let Some(source_t) = src_tokens_iter.next() {
                        // lost sight
                        if Self::check_tokens_eq_verilog(source_t.as_type(), find_t.as_type())
                            == false
                        {
                            tracking = false;
                            break;
                        }
                    } else {
                        tracking = false;
                        break;
                    }
                }
                // initiate lock
                if tracking == true {
                    // @todo: return last index of list for skipping purposes
                    return Some(t.locate().clone());
                }
            }
            // @todo: handle follow better (keep iterating until hitting a wrong one)
        }
        None
    }
}

// SystemVerilog support
impl Read {
    fn read_systemverilog(
        &self,
        contents: &str,
    ) -> Result<(Option<Position>, Option<Position>), Fault> {
        let start: Option<SystemVerilogTokenizer> = match &self.start {
            Some(s) => Some(SystemVerilogTokenizer::from_str(s)?),
            None => None,
        };
        let end: Option<SystemVerilogTokenizer> = match &self.end {
            Some(s) => Some(SystemVerilogTokenizer::from_str(s)?),
            None => None,
        };
        let comment: Option<SystemVerilogTokenizer> = match &self.comment {
            Some(s) => Some(SystemVerilogTokenizer::from_str(s)?),
            None => None,
        };

        let src_tokens = SystemVerilogTokenizer::from_source_code(&contents).into_tokens_all();

        // perform a search on tokens
        let (start, end) = {
            // get the tokens
            let start_tokens = match &start {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &SystemVerilogToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let end_tokens = match &end {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &SystemVerilogToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let comment_tokens = match &comment {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &SystemVerilogToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            // search over the source code tokens
            let start = Self::find_location_systemverilog(&src_tokens, &start_tokens);
            // limit based on starting index
            let remaining_tokens = match &start {
                Some(pos) => src_tokens
                    .into_iter()
                    .skip_while(|p| p.locate() <= pos)
                    .collect(),
                None => {
                    if self.start.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'start' code chunk"
                        )))?;
                    }
                    src_tokens
                }
            };

            let end = Self::find_location_systemverilog(&remaining_tokens, &end_tokens);
            let remaining_tokens = match &end {
                Some(pos) => remaining_tokens
                    .into_iter()
                    .take_while(|p| p.locate() < pos)
                    .collect(),
                None => {
                    if self.end.is_some() == true {
                        return Err(AnyError(format!(
                            "failed to find code segment matching 'end' code chunk"
                        )))?;
                    }
                    remaining_tokens
                }
            };

            // find the comment
            let first_token = Self::find_location_systemverilog(&remaining_tokens, &comment_tokens);
            // grab all continuous '--' tokens immediately above/before this location
            let comment = match &first_token {
                Some(pos) => {
                    let mut line = pos.line();
                    // println!("{:?}", pos);
                    match remaining_tokens
                        .into_iter()
                        .rev()
                        .skip_while(|p| p.locate() >= pos)
                        .take_while(|p| {
                            // only take immediate comments grouped together
                            line -= 1;
                            p.as_type().as_comment().is_some() && p.locate().line() == line
                        })
                        .last()
                    {
                        Some(token) => Some(token.locate().clone()),
                        None => {
                            return Err(AnyError(format!(
                                "Zero comments associated with code chunk"
                            )))?
                        }
                    }
                }
                None => {
                    if self.comment.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'doc' code chunk"
                        )))?;
                    }
                    None
                }
            };

            let end = match &comment {
                Some(_) => first_token,
                None => end,
            };

            let start = match comment {
                Some(c) => Some(c),
                None => start,
            };

            (start, end)
        };
        Ok((start, end))
    }

    fn check_tokens_eq_systemverilog(
        source: &SystemVerilogToken,
        sub: &SystemVerilogToken,
    ) -> bool {
        match sub {
            // skip EOF token
            &SystemVerilogToken::EOF => true,
            // only match on the fact that they are comments
            SystemVerilogToken::Comment(_) => source.as_comment().is_some(),
            _ => source == sub,
        }
    }

    fn find_location_systemverilog(
        src_tokens: &Vec<Token<SystemVerilogToken>>,
        find_tokens: &Vec<&Token<SystemVerilogToken>>,
    ) -> Option<Position> {
        let mut tracking: bool;
        let mut src_tokens_iter = src_tokens.iter();
        while let Some(t) = src_tokens_iter.next() {
            // begin to see if we start tracking
            // println!("{:?} {:?}", find_tokens.first().unwrap().as_type(), t.as_type());

            if find_tokens.len() > 0
                && Self::check_tokens_eq_systemverilog(
                    t.as_type(),
                    find_tokens.first().unwrap().as_type(),
                ) == true
            {
                // println!("{}", "HERE");
                let mut find_tokens_iter = find_tokens.iter().skip(1);
                tracking = true;
                while let Some(find_t) = find_tokens_iter.next() {
                    // skip the EOF token
                    if find_t.as_type() == &SystemVerilogToken::EOF {
                        continue;
                    }
                    if let Some(source_t) = src_tokens_iter.next() {
                        // lost sight
                        if Self::check_tokens_eq_systemverilog(source_t.as_type(), find_t.as_type())
                            == false
                        {
                            tracking = false;
                            break;
                        }
                    } else {
                        tracking = false;
                        break;
                    }
                }
                // initiate lock
                if tracking == true {
                    // @todo: return last index of list for skipping purposes
                    return Some(t.locate().clone());
                }
            }
            // @todo: handle follow better (keep iterating until hitting a wrong one)
        }
        None
    }
}
