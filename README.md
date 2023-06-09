![](https://github.com/kaindljulian/sat_solver/actions/workflows/build_and_test.yml/badge.svg)

__Compile and Run:__

````
$ cargo build --release
$ ./target/release/sat_cli ./test_formulas/add4.unsat
````
__Help:__
```
Usage: sat_cli <FILE>

Arguments:
  <FILE>  A dimacs cnf file

Options:
      --no-dlis  Disable DLIS decision heuristic
  -h, --help     Print help
```
