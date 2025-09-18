A reimplementation of Walter Fontana's Alchemy. Pipe lambda expressions into 
`stdin` to start a default simulation. 

Usage:

`alchemy`

Build: 

`cargo build`

Testing:

`cargo run -- {args}`

With the binary tree generators from the 
[lambda-btree](https://github.com/AgentElement/lambda-btree) crate, you can
run a simple alchemy simulation with the following command.

`python /path/to/src/fontana_generator.py | cargo run -- {args}`


Documentation:

* Full documentation: `cargo doc --open`
* Help: `cargo run -- --help`

The documentation for the configuration file is in the `Config` object.


# Interactive Dashboard

Can be found here: https://github.com/mathis-group/Alchemy-Dashboard