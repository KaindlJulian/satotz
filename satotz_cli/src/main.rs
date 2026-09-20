use clap::Parser;
use satotz_lib::cnf::CNF;
use satotz_lib::solver::Solver;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[group(required = true)]
struct Args {
    /// A dimacs cnf file
    file: PathBuf,

    /// Disable DLIS decision heuristic
    #[arg(long)]
    no_dlis: bool,

    /// Write the NDJSON event log to this file
    #[arg(long, value_name = "FILE")]
    events: Option<PathBuf>,

    /// Also log an inspect event per clause BCP looks at
    #[arg(long, requires = "events")]
    events_bcp: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let cnf = match CNF::from_file(args.file.clone()) {
        Ok(cnf) => cnf,
        Err(e) => return fail(&format!("{}: {e}", args.file.display())),
    };

    let mut solver = Solver::from_cnf(cnf);

    if args.no_dlis {
        solver = solver.without_dlis();
    }

    if let Some(path) = &args.events {
        if let Err(e) = solver.open_event_log(path, args.events_bcp) {
            return fail(&format!("cannot write {}: {e}", path.display()));
        }
    }

    let sat = solver.solve();

    if let Err(e) = solver.close_event_log() {
        return fail(&format!("writing events: {e}"));
    }

    if sat {
        println!("s SATISFIABLE");
        println!("v {:?}", solver.assignment());
        let _ = io::stdout().flush();
        ExitCode::from(10)
    } else {
        println!("s UNSATISFIABLE");
        let _ = io::stdout().flush();
        ExitCode::from(20)
    }
}

fn fail(message: &str) -> ExitCode {
    eprintln!("satotz: {message}");
    ExitCode::from(1)
}
