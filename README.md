# verilot

verilot (verifiable lottery) is a command line tool for running and verifying one-time lotteries.

## Install

Install Rust and Cargo with [Rustup](http://rustup.rs).

Install verilot with `cargo install verilot`.

## Usage

The verifiable lottery process has a few overall steps.

1. Generate secret random data and publish commitment.
1. Gather entrants.
1. Run lottery.
1. Publish results (including secret random data).
1. Users verify results.

### Generating Secret

```
verilot generate /path/to/secret.txt > /path/to/commitment.txt
```

`generate` will write the secret bytes (hex-encoded) to the path given in the argument.
It will output the commitment to stdout.
We recommend publishing the commitment for the participants in your lottery to verify with.

If you publish the secret, users can use compute power to increase their chances of winning the lottery.

### Running Lottery

```
printf "foo\nbar\nbaz" | verilot lottery --secret /path/to/secret.txt > /path/to/results.json
```

`lottery` takes the list of entrants as newline-separated arbitrary strings as input on stdin.
It outputs a json structure with the outcome to stdout.
The whole json file should be published for entrants to verify fairness.

The output contains a field of the entrants in order of hash value.
Winners are the entrants with the lowest hash value.

#### Hashing Input

Since it's common to not want to publish the raw entrant input strings (which is required for verifying fairness), we provide a subcommand to hash raw input strings.

```
printf "foo\nbar\nbaz" | verilot digest

# Output:
# abcdef... "foo"
# abcdef... "bar"
# abcdef... "baz"
```

You can then easily split out the hash with `cut -f1 -d' '`.
You'll need to manually pair the hashes in the results with the output of `digest` to get the appropriate winners.

### Verifying

```
cat /path/to/results.json | verilot verify --commitment "abcdef"
```

`verify` takes the json results of the lottery as input on stdin.
It optionally takes the commitment as hex string.
Verify will exit with a non-zero code if the verification fails.

## Algorithm

Each entrant gets a "ticket" calculated as `Hash(entrant_data | secret)`.
Winners are the entrants with the lowest tickets.
