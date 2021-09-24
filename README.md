# edr\_test

This is a simple framework for simulating basic user activity, in order to test
user activity monitoring systems, such as Red Canary EDR.

# Description

There are essentially two components, a registry of "activities", which includes
every activity that can be simulated (e.g. file creation, network access), and a
simple command syntax to trigger these activities.  Here is an example (located
in `examples/file.act`):

```
# simulate file activity

dir = "tmp/test"
file = join dir "/hello"

file.create :dir dir
file.create :file file
```

This creates the directory "tmp/test", and then creates the file
"tmp/test/hello".

The program's execution essentially follows this sequence:
- Parse command line arguments
- Register activities
- Execute script

Additionally, each activity causes a log record to be emitted on success, which
will be added to the log file in json format.

# Script Syntax

Comments start with `#` and end at the end of the line.

Variables are always strings, `:XXX` is shorthand for `"XXX"`.

Statements are terminated by newlines.  Two statements are allowed:
- `id = func [id [id [...]]] | id`
- `func [id [id [...]]]`

# Installation

Assuming you have `cargo` installed:

```
git clone https://github.com/annacrombie/edr_test.git && cd edr_test
cargo build
```

The `edr_test` binary should now be located at `target/debug/edr_test`.

You may optionally run `cargo test -- --test-threads=1` to run the tests.  Not
all functionality has been tested due to time constraints.

# Usage

See `edr_test -h`
