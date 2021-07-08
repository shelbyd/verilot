use blake2::{Blake2b, Digest};
use generic_array::GenericArray;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    error::Error,
    fmt,
    fs::File,
    io::{BufRead, Read, Write},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Generate {
        secret_out: PathBuf,
    },
    Lottery {
        #[structopt(long)]
        secret: PathBuf,
    },
    Verify {
        #[structopt(long)]
        commitment: Option<String>,
    },
}

type DynResult<T> = Result<T, Box<dyn Error>>;

fn main() -> DynResult<()> {
    let options = Options::from_args();

    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    match options.command {
        Command::Generate { secret_out } => {
            let mut out = File::create(&secret_out)?;
            do_generate(&mut out, &mut stdout)
        }
        Command::Lottery { secret } => {
            let secret = std::fs::read(&secret)?;
            do_lottery(&secret, stdin.lock(), &mut stdout)
        }
        Command::Verify { commitment } => do_verify(stdin, commitment.as_ref()),
    }
}

fn do_generate(secret_out: &mut impl Write, stdout: &mut impl Write) -> DynResult<()> {
    use rand::RngCore;

    let mut rng = rand::thread_rng();
    let mut random_bytes = [0; 64];
    rng.fill_bytes(&mut random_bytes);

    let hex_bytes = hex::encode(&random_bytes);
    secret_out.write(hex_bytes.as_bytes())?;

    let commitment = Hash::of(&[&random_bytes]);
    writeln!(stdout, "{}", commitment.to_string())?;

    Ok(())
}

fn do_lottery(secret: &[u8], stdin: impl BufRead, stdout: &mut impl Write) -> DynResult<()> {
    let entries = stdin.lines().collect::<Result<Vec<_>, _>>()?;

    let outcome = Outcome::generate(entries, hex::decode(secret)?);
    serde_json::to_writer_pretty(stdout, &outcome)?;

    Ok(())
}

fn do_verify(stdin: impl Read, commitment: Option<&String>) -> DynResult<()> {
    let outcome: Outcome = serde_json::from_reader(stdin)?;

    outcome.verify();

    if let Some(commitment) = commitment {
        let hash = Hash::parse(commitment)?;
        outcome.verify_commitment(hash);
    }

    Ok(())
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Debug)]
struct Hash(GenericArray<u8, <Blake2b as Digest>::OutputSize>);

impl Hash {
    fn of(sequences: &[&[u8]]) -> Self {
        let mut hasher = Blake2b::new();
        for bytes in sequences {
            hasher.update(bytes);
        }
        Hash(hasher.finalize())
    }

    fn parse(s: &str) -> DynResult<Self> {
        Ok(serde_json::from_str(&format!("\"{}\"", s))?)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> serde::de::Visitor<'de> for StringVisitor {
            type Value = Hash;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 64 byte hex string")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let bytes = hex::decode(s).map_err(E::custom)?;
                let array =
                    GenericArray::from_exact_iter(bytes).ok_or(E::custom("incorrect length"))?;
                Ok(Hash(array))
            }
        }

        deserializer.deserialize_str(StringVisitor)
    }
}

impl ToString for Hash {
    fn to_string(&self) -> String {
        hex::encode(self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Outcome {
    ordered_entries: Vec<(Hash, String)>,
    secret: Vec<u8>,
}

impl Outcome {
    fn generate(entries: Vec<String>, secret: Vec<u8>) -> Outcome {
        let mut hashed_entries = entries
            .into_iter()
            .map(|entry| (Hash::of(&[&entry.as_bytes(), &secret]), entry))
            .collect::<Vec<_>>();
        hashed_entries.sort();
        Outcome {
            ordered_entries: hashed_entries,
            secret,
        }
    }

    fn verify(&self) {
        let entries = self.ordered_entries.iter().map(|(_, e)| e).cloned().collect();
        let generated = Outcome::generate(entries, self.secret.clone());
        assert_eq!(self, &generated);
    }

    fn verify_commitment(&self, hash: Hash) {
        let expected = Hash::of(&[&self.secret]);
        assert_eq!(hash, expected);
    }
}
