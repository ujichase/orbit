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

use colored::Colorize;
use std::{fmt::Display, path::PathBuf};

use crate::core::{
    blueprint::Scheme,
    ip::IpSpec,
    lang::{lexer::Position, LangIdentifier},
    pkgid::PkgPart,
    version::{AnyVersion, PartialVersion, Version},
    visibility::Visibility,
};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
    #[error("an ip already exists at {0:?}")]
    IpExistsAtPath(PathBuf),
    #[error("path {0:?} already exists {1}")]
    PathAlreadyExists(PathBuf, Hint),
    #[error("directory {0:?} is an invalid ip name: {1}{2}")]
    CannotAutoExtractNameFromPath(String, LastError, Hint),
    #[error("file system path {0:?} is missing a name{1}")]
    MissingFileSystemPathName(PathBuf, Hint),
    #[error("failed to create new ip: {0}")]
    FailedToCreateNewIp(LastError),
    #[error("failed to initialize ip: {0}")]
    FailedToInitIp(LastError),
    #[error("a target must be defined")]
    MissingRequiredTarget,
    #[error("command must be ran from a local ip: no ip found in current directory or any parent directory")]
    NoWorkingIpFound,
    #[error("command must be ran from a local ip when an ip is not explicitly defined: no ip found in current directory or any parent directory")]
    NoAssumedWorkingIpFound,
    #[error("ip {0:?} does not exist in the cache")]
    IpNotFoundInCache(String),
    #[error("ip {0:?} does not exist in the catalog{1}")]
    IpNotFoundAnywhere(String, Hint),
    #[error("exited with error code: {0}")]
    ChildProcErrorCode(i32),
    #[error("terminated by signal")]
    ChildProcTerminated,
    #[error("no target named {0:?}{1}")]
    TargetNotFound(String, Hint),
    #[error("a target must be specified{0}")]
    TargetNotSpecified(Hint),
    #[error("failed to execute target process: {0}")]
    TargetProcFailed(LastError),
    #[error("failed to execute protocol process: {0}")]
    ProtocolProcFailed(LastError),
    #[error("no protocol named {0:?}")]
    ProtocolNotFound(String),
    #[error("failed to modify configuration: {0}")]
    ConfigNotSaved(LastError),
    #[error("configuration field {0:?} does not store a list")]
    ConfigFieldNotList(String),
    #[error("failed to process value {1:?} for configuration field \"include\" at {0:?}: {2}")]
    ConfigIncludeFailed(String, String, LastError),
    #[error("failed to load configuration file at {0:?}: {1}")]
    ConfigLoadFailed(String, LastError),
    #[error("failed to save configuration file at {0:?}: {1}")]
    ConfigSaveFailed(String, LastError),
    #[error("failed to parse source code file {0:?}: {1}")]
    SourceCodeInvalidSyntax(PathBuf, LastError),
    #[error("failed to process ip graph: {0}")]
    IpGraphFailed(LastError),
    #[error("failed to parse identifier: {0}")]
    CrossIdentifierParsingFailed(LastError),
    #[error("duplicate identifier \"{0}\" found in the following source files:\n\n  location 1: {1}{2}\n  location 2: {3}{4}{5}")]
    DuplicateIdentifiersCrossLang(String, String, Position, String, Position, Hint),
    #[error(
        "blueprint plan \"{0}\" not supported by the current target; supported plans are: {1:?}"
    )]
    BlueprintPlanNotSupported(Scheme, Vec<Scheme>),
    #[error("blueprint plan \"{0}\" not supported by the current target; no plans are defined so it can only accept \"{1}\"")]
    BlueprintPlanMustBeDefault(Scheme, Scheme),
    #[error("failed to find unit with matching name \"{0}\"{1}")]
    GetUnitNotFound(String, Hint),
    #[error("unit \"{0}\" is not a usable design component{1}")]
    GetUnitNotComponent(String, Hint),
    #[error("failed to load ip: {0}")]
    IpLoadFailed(LastError),
    #[error("failed to parse ip name: {0}")]
    IpNameParseFailed(LastError),
    #[error("listed version {0} does not match ip's actual version {1}")]
    DependencyIpRelativeBadVersion(PartialVersion, Version),
    #[error("listed name {0} does not match ip's actual name {1}")]
    DependencyIpRelativeBadName(PkgPart, PkgPart),
    #[error("failed to load lockfile: {0}")]
    LockfileLoadFailed(LastError),
    #[error("failed to install: {0}")]
    InstallFailed(LastError),
    #[error("ip has dependencies that are relative")]
    IpHasRelativeDependencies,
    #[error("a testbench is required to test")]
    TestbenchRequired,
    #[error("top \"{0}\" is not tested in testbench \"{1}\"{2}")]
    TopNotInTestbench(LangIdentifier, LangIdentifier, Hint),
    #[error("lockfile entry \"{0}\" is not queued for installation (missing download)")]
    EntryMissingDownload(IpSpec),
    #[error("lockfile entry \"{0}\" is not queued for installation")]
    EntryNotQueued(IpSpec),
    #[error("lockfile entry \"{0}\" is not queued for installation (unknown ip)")]
    EntryUnknownIp(IpSpec),
    #[error("found {0} ips downloaded as candidates: {1}{2}")]
    DownloadFoundManyIps(usize, String, Hint),
    #[error("lockfile is missing or out of date{0}")]
    PublishMissingLockfile(Hint),
    #[error("the ip manifest's source field is required to publish, but is undefined")]
    PublishMissingSource,
    #[error("ip {0} is already published to at least one of the specified channels")]
    PublishAlreadyExists(IpSpec),
    #[error("default channel \"{0}\" does not exist")]
    DefChanNotFound(String),
    #[error("listed channel \"{0}\" does not exist")]
    ChanNotFound(String),
    #[error("a channel is required to publish an ip")]
    NoChanDefined,
    #[error("failed to build hdl graph: {0}")]
    PublishHdlGraphFailed(LastError),
    #[error("ip {0} is ready to be published{1}")]
    PublishDryRunDone(IpSpec, Hint),
    #[error("checksums do not match between downloaded ip and local ip{0}")]
    PublishChecksumsOff(Hint),
    #[error("channel's resolved path {0:?} does not exist")]
    ChannelPathNotFound(PathBuf),
    #[error("channel's resolved path {0:?} is not a directory")]
    ChannelPathNotDir(PathBuf),
    #[error("ip has \"{0}\" listed as a relative dependency")]
    PublishRelativeDepExists(PkgPart),
    #[error("failed to pass publish checkpoint: {0}")]
    PublishFailedCheckpoint(LastError),
    #[error("cyclic dependency with local ip \"{0}\"")]
    CyclicDependencyIp(PkgPart),
    #[error("failed to get uuid for ip \"{0}\" due to missing or corrupted lockfile{1}")]
    RequiredUuuidMissing(IpSpec, Hint),
    #[error("failed to find a version matching \"{0}\"{1}")]
    VersionNotFound(AnyVersion, Hint),
    #[error("cannot {0} unit \"{1}\" due to {2} visibility{3}")]
    UnitIsWrongVisibility(String, LangIdentifier, Visibility, Hint),
    #[error("path {0:?} is not a configuration file{1}")]
    ConfigBadPath(PathBuf, Hint),
    #[error("invalid key: \"include\" is not allowed in non-global configuration file")]
    ConfigIncludeInNonglobal,
    #[error("expects {0} characters but found {1}")]
    UuidWrongSize(usize, usize),
    #[error("invalid character \"{0}\" does not belong to alphabet (a-z0-9)")]
    UuidInvalidChar(char),
    #[error(
        "ip namespace collision for \"{0}\": please disambiguate by providing the appropriate uuid"
    )]
    IpNamespaceCollision(String),
    #[error(
        "uuid for ip \"{0}\" has been modified which can result in unintended consequences{1}"
    )]
    UuidModified(PkgPart, Hint),
    #[error("failed to get current working directory (does it still exist?)")]
    FailedToGetCurDir,
    #[error(
        "failed to detect user's home directory; please set the ORBIT_HOME environment variable"
    )]
    FailedToGetHomeDir,
    #[error("directory {0:?} does not exist for ORBIT_HOME")]
    OrbitHomeDoesNotExist(PathBuf),
    #[error("edge kinds are: \"unit\", \"ip\", \"all\"")]
    EdgeKindInvalid(String),
}

#[derive(Debug, PartialEq)]
pub struct LastError(pub String);

impl Display for LastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Error::lowerize(self.0.to_string()))
    }
}

impl Error {
    pub fn lowerize(s: String) -> String {
        // get the first word
        let first_word = s.split_whitespace().into_iter().next().unwrap();
        // retain punctuation if the first word is all-caps and longer than 1 character
        if first_word.len() > 1
            && first_word
                .chars()
                .find(|c| c.is_ascii_lowercase() == true)
                .is_none()
        {
            s.to_string()
        } else {
            s.char_indices()
                .map(|(i, c)| if i == 0 { c.to_ascii_lowercase() } else { c })
                .collect()
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Hint {
    TargetsList,
    PublishSyncRemote,
    CatalogList,
    InitNotNew,
    IpNameSeparate,
    ResolveDuplicateIds1,
    ResolveDuplicateIds2,
    ShowAvailableUnitsLocal,
    ShowAvailableUnitsExternal(IpSpec),
    DutSpecify,
    WantsTestbench,
    WantsTop,
    TopSpecify,
    BenchSpecify,
    RootSpecify,
    IncludeAllInPlan,
    SpecifyIpSpecForDownload,
    MakeLock,
    PublishWithReady,
    RegenerateLockfile,
    ShowVersions,
    ShowConfigFiles,
    ConfirmUuidChange(String),
}

impl Display for Hint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mixed_prompt = match self {
            Self::ShowAvailableUnitsExternal(spec) => Some(format!(
                "use `orbit info {0} --units` to display available units",
                spec
            )),
            _ => None,
        };
        let message = match self {
            Self::CatalogList => "use `orbit search` to see the list of known ips",
            Self::TargetsList => "use `orbit build --list` to see the list of defined targets",
            Self::InitNotNew => "use `orbit init` to initialize an existing directory",
            Self::IpNameSeparate => {
                "use the \"--name\" option for making an ip name separate from the directory name"
            }
            Self::ResolveDuplicateIds1 => HINT_1,
            Self::ResolveDuplicateIds2 => HINT_2,
            Self::ShowAvailableUnitsLocal => "use `orbit info --units` to display available units",
            Self::ShowAvailableUnitsExternal(_) => mixed_prompt.as_ref().unwrap(),
            Self::DutSpecify => "use the \"--dut\" option to specify the design under test",
            Self::WantsTestbench => {
                "use `orbit test` and its \"--tb\" option to select testbenches"
            }
            Self::WantsTop => {
                "use `orbit build` and its \"--top\" option to select top-level designs"
            }
            Self::TopSpecify => "use the \"--top\" option to specify the top-level design",
            Self::BenchSpecify => "use the \"--tb\" option to specify the testbench",
            Self::RootSpecify => "use the \"--root\" option to specify the root design unit",
            Self::IncludeAllInPlan => "use the \"-all\" flag to continue with this setup",
            Self::SpecifyIpSpecForDownload => {
                "consider providing the ip specification for the requested ip to download"
            }
            Self::MakeLock => "use `orbit lock` to generate the latest lockfile for this ip",
            Self::PublishWithReady => "use the \"--ready\" flag to publish the ip to its channels",
            Self::RegenerateLockfile => "verify the ip's lockfile exists and is up to date",
            Self::ShowVersions => "use `orbit info <ip> --versions` to see all known versions",
            Self::ShowConfigFiles => {
                "use `orbit config --list` to see the list of current configuration files"
            }
            Self::PublishSyncRemote => {
                "check that the local ip's contents matches the source's contents"
            }
            Self::ConfirmUuidChange(uuid) => &format!(
                "resolve this error by either
    1) using `orbit lock --force` to keep the new uuid
    2) copy the original uuid \"{0}\" back into the manifest to keep the old uuid",
                uuid
            ),
        };
        write!(
            f,
            "\n\n{}: {}",
            "hint".green(),
            Error::lowerize(message.to_string())
        )
    }
}

const HINT_1: &str = "resolve this error by either
    1) renaming one of the units to a unique identifier
    2) adding one of the file paths to the manifest's \"ip.exclude\" field";

const HINT_2: &str = "resolve this error by either
    1) renaming the unit in the local ip to a unique identifier
    2) removing the direct dependency from Orbit.toml
    3) adding the file path for the local ip's unit to the manifest's \"ip.exclude\" field";
