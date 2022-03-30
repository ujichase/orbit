use orbit::interface::cli::*;
use orbit::interface::errors::*;
use orbit::interface::command::*;
use orbit::interface::arg::*;

fn main() {
    match Add::from_cli(&mut Cli::tokenize(std::env::args())) {
        Ok(r) => r.exec(),
        Err(e) => eprintln!("error: {}", e),
    }
}

// demo program
#[derive(Debug, PartialEq)]
struct Add {
    lhs: u32,
    rhs: u32,
    verbose: bool,
}

impl Command for Add {
    fn exec(self) -> () {
        println!("{}", self.run());
    }
}

impl Add {
    /// Simple fn to return an answer for the `Add` test command.
    fn run(self) -> String {
        let sum = self.lhs + self.rhs;
        match self.verbose {
            true => format!("{} + {} = {}", self.lhs, self.rhs, sum),
            false => format!("{}", sum),
        }
    }
}

impl FromCli for Add {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        Ok(Add {
            verbose: cli.check_flag(Flag::new("verbose"))?,
            lhs: cli.require_positional(Positional::new("lhs"))?,
            rhs: cli.require_positional(Positional::new("rhs"))?,
        })
    }
}