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

use colored::*;

#[allow(unused_must_use)]
fn main() -> () {
    let rc = match install() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            101
        }
    };
    // allow user to see final messages before closing the window
    poll_response("press enter to exit ... ");
    std::process::exit(rc as i32);
}

fn install() -> Result<(), Box<dyn std::error::Error>> {
    // route operating system accordingly
    if cfg!(unix) {
        unix()
    } else if cfg!(windows) {
        windows()
    } else {
        Err(InstallError::UnknownFamily)?
    }
}

use home::home_dir;
use orbit::util::filesystem;
use orbit::util::prompt;

#[cfg(target_os = "windows")]
const EXE_NAME: &str = "orbit.exe";
#[cfg(not(target_os = "windows"))]
const EXE_NAME: &str = "orbit";

const EXE_DIR: &str = ""; // "bin/""

/// unix installation steps (copies only the binary)
fn unix() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", HEADER);

    // gather path for what will be installed
    let contents = {
        // fetch this program's (installer) path
        let mut root = filesystem::get_exe_path()?;
        // remove file to get parent directory
        root.pop();
        root.join(EXE_DIR.to_owned() + EXE_NAME)
    };

    // verify this program was could find the executable
    if contents.exists() == false {
        return Err(InstallError::UndetectedExe(contents))?;
    }

    // 1. compute installation size
    let megabytes = fs_extra::dir::get_size(&contents)? as f32 / 1000000 as f32;
    println!("installation size: {:.2} MB", megabytes);

    // 2. configure installation destination
    let path = PathBuf::from("/usr/local/bin");
    let path = installation_path(path)?;

    let dest = path.join(EXE_NAME);

    // check if a file named "orbit" already exists
    if dest.exists() == true
        && prompt::prompt(&format!(
            "file {} already exists; is it okay to replace it",
            dest.display()
        ))? == false
    {
        println!("cancelled installation");
        return Ok(());
    }

    // 3. ask user for permission
    match prompt::prompt("Install")? {
        true => {
            // 4a. copy the binary to the location
            std::fs::copy(contents, dest)?;
            println!("successfully installed orbit");
        }
        false => {
            // 4b. cancel installation
            println!("cancelled installation")
        }
    };
    Ok(())
}

/// windows installation steps (copies the entire orbit download folder)
fn windows() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", HEADER);

    // gather path for what will be installed (@note: this is a folder of the contents for windows-style install)
    let contents = {
        let mut root = filesystem::get_exe_path()?;
        // get base directory from where installer exists
        root.pop();
        root
    };

    // verify this program was could find the executable
    if contents.join(EXE_DIR.to_owned() + EXE_NAME).exists() == false {
        return Err(InstallError::UndetectedExe(
            contents.join(EXE_DIR.to_owned() + EXE_NAME),
        ))?;
    }

    // 1. compute installation size
    let megabytes =
        orbit::util::filesystem::compute_size(&contents, orbit::util::filesystem::Unit::MegaBytes)?;
    println!("installation size: {:.2} MB", megabytes);

    // 2. configure installation destination
    let path = match std::env::var("LOCALAPPDATA") {
        Ok(v) => std::path::PathBuf::from(v).join("Programs"),
        Err(_) => std::path::PathBuf::from(home_dir().unwrap()),
    };
    let path = installation_path(path)?;

    let dest = path.join("orbit");

    // check if a folder named "orbit" already exists
    if dest.exists() == true
        && prompt::prompt(&format!(
            "directory {} already exists; is it okay to replace it",
            dest.display()
        ))? == false
    {
        println!("cancelled installation");
        return Ok(());
    }

    // 3. ask user for permission
    match prompt::prompt("Install")? {
        true => {
            // 4a. copy the binary to the location
            let options = {
                let mut opt = fs_extra::dir::CopyOptions::new();
                opt.content_only = true;
                opt
            };
            // remove old folder called 'orbit'
            if dest.exists() == true {
                std::fs::remove_dir_all(&dest)?;
            }
            // recreate 'orbit' folder
            std::fs::create_dir(&dest)?;
            // copy contents (installed directory) to renewed orbit directory destination
            fs_extra::dir::copy(&contents, &dest, &options)?;
            println!("successfully installed orbit");
            println!(
                "{} add {} to the user PATH variable to call `orbit` from the command-line",
                "tip:".blue().bold(),
                path.join("orbit".to_owned() + &{ if EXE_DIR.len() > 0 { format!("/{}", EXE_DIR) } else { String::new() } }).display()
            );
        }
        false => {
            // 4b. cancel installation
            println!("cancelled installation")
        }
    }
    Ok(())
}

use fs_extra;
use std::path::PathBuf;

fn installation_path(path: PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    println!("default installation path: {}", path.display());
    let path = match poll_response("enter installation path or press enter to continue: ")? {
        Some(r) => PathBuf::from(r),
        None => path,
    };
    // verify path exists
    match path.exists() {
        true => {
            println!("set installation path: {}", path.display());
            Ok(path)
        }
        false => Err(InstallError::PathDNE(path))?,
    }
}

#[derive(Debug, PartialEq)]
enum InstallError {
    UnknownFamily,
    PathDNE(PathBuf),
    UndetectedExe(PathBuf),
}

impl std::error::Error for InstallError {}

use std::fmt::Display;

impl Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::PathDNE(p) => write!(f, "path {:?} does not exist", p),
            Self::UnknownFamily => write!(f, "unknown family (did not detect unix or windows)"),
            Self::UndetectedExe(p) => write!(
                f,
                "installer program failed to find executable path {:?}",
                p
            ),
        }
    }
}

use std::io;
use std::io::Write;

fn poll_response(msg: &str) -> Result<Option<String>, Error> {
    print!("{}", msg);
    std::io::stdout().flush()?;
    let resp = capture_response(&mut io::stdin().lock())?;
    Ok(match resp.trim().is_empty() {
        true => None,
        false => Some(resp),
    })
}

use std::io::{Error, Read};

fn capture_response(input: &mut (impl Read + std::io::BufRead)) -> Result<String, Error> {
    let mut buffer: String = String::new();
    input.read_line(&mut buffer)?;
    Ok(buffer.trim_end().to_string())
}

const HEADER: &str = "\
------------------------------------------------------------
::              ORBIT INSTALLATION PROGRAM                ::
------------------------------------------------------------
";

#[cfg(test)]
mod test {

    #[test]
    fn names() {
        let n = std::path::PathBuf::from("c:/users/chase/orbit-1.0.0/");
        assert_eq!(n.file_name().unwrap(), "orbit-1.0.0");

        let n = std::path::PathBuf::from("c:/users/chase/orbit-1.0.0");
        assert_eq!(n.file_name().unwrap(), "orbit-1.0.0");
    }
}
