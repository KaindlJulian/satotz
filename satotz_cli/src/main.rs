use clap::Parser;
use satotz_lib::cnf::CNF;
use satotz_lib::solver::Solver;
use std::path::PathBuf;

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

fn main() {
    let args = Args::parse();
    let cnf = CNF::from_file(args.file);
    let mut solver = Solver::from_cnf(cnf);

    if args.no_dlis {
        solver = solver.without_dlis();
    }

    if let Some(path) = &args.events {
        if let Err(e) = solver.open_event_log(path, args.events_bcp) {
            eprintln!("satotz: cannot write {}: {e}", path.display());
            std::process::exit(1);
        }
    }

    let sat = solver.solve();

    if let Err(_) = solver.close_event_log() {
        std::process::exit(1);
    }

    if sat {
        println!("s SATISFIABLE");
        println!("v {:?}", solver.assignment());
        std::process::exit(10);
    } else {
        println!("s UNSATISFIABLE");
        std::process::exit(20);
    }
}
