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

Example output:

```
{
  "ordered_entries": [
    {
      "entrant": "baz",
      "ticket": "030b0be9796353fbadd6d4acb1883551f016b30316b7510b1525642689b9f32c3ada75bb4ba005a4abcf61a8e09ec0c8a2410a0415d0a9642e08b622290a49c8"
    },
    {
      "entrant": "bar",
      "ticket": "4bd8e47093eec63086e4c1f3dc717b4c7724e96b30ff1aa5b498dc1eec8fae461eb81dd6ecedb208bc166f50dc260c9e9f60d3a2cf1e8a15a8f3cba36fd3b247"
    },
    {
      "entrant": "foo",
      "ticket": "9f846275d1b9f2eb9e4c7e830ff33f075c38b7ae84ae8eabe9c4a04ec026ff4bb443b57a5324c0f498adf98c2a5fd35cd638381eaa784ae262248d3e084b1f8e"
    }
  ],
  "secret": [
    43, 59, 138, 218, 23, 151, 103, 84,
    227, 162, 246, 113, 150, 225, 237, 223,
    14, 248, 110, 47, 197, 241, 70, 196,
    213, 113, 253, 230, 211, 167, 112, 94,
    150, 108, 15, 99, 237, 129, 217, 131,
    138, 198, 202, 144, 131, 242, 157, 17,
    46, 14, 55, 107, 185, 58, 38, 40,
    64, 139, 62, 107, 234, 215, 85, 237
  ]
}
```

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

verilot uses Blake2b for all hashing.
`generate` generates secrets with 512 bits of randomness from the OS's secure pRNG.
See https://docs.rs/rand/0.8.4/rand/rngs/struct.ThreadRng.html for more details.
