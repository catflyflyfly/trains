# Trains

- [Trains](#trains)
  - [Algorithms](#algorithms)
  - [Usage](#usage)
  - [License](#license)

This is an application for solving trains problems.

This codebase includes:

- CLI Application module
- main application module
- test cases, and test module
- example scripts
- solution progress, tracked by git history

## Algorithms

1. Parse the input from the cli to create the model. See modules: `args`, `model`
2. Build a shortest path map using dijkstra's between every stations. See modules: `model::route_path`
3. List all possible actions includings picking and dropping every packages.
4. Find a shortest path to complete every actions using dijkstra's. See modules: `state`

## Usage

This is a CLI application. You need to use the terminal to execute it.

The application binary is at `bin/trains`. For help, run:

```sh
bin/trains --help
```

Since input parsing error handling is not implemented nicely due to time constraint, you should try one of the examples by executing scripts in `scripts` dir (including the example input of the problem)

```sh
scripts/example.sh
```

To run test, you need `cargo` to run:

```sh
cargo test
```

## License

This project is licensed under the terms of the MIT license.
