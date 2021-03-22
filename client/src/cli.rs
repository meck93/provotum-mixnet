use clap::Clap;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap, Debug)]
#[clap(
    name = "provotum",
    version = "1.0",
    author = "Moritz Eck <moritz.eck@gmail.com>"
)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    #[clap(name = "voter")]
    Voter(Voter),
    #[clap(name = "va")]
    VotingAuthority(VotingAuthority),
    #[clap(name = "sealer")]
    Sealer(Sealer),
}

/// A subcommand for controlling the Voter
#[derive(Clap, Debug)]
pub struct Voter {
    /// The name of the vote
    #[clap(short, long)]
    pub vote: String,
    /// The name of the question
    #[clap(short, long)]
    pub question: String,
    /// The number of votes to create
    #[clap(long)]
    pub nr_of_votes: usize,
    /// The set of allowed votes
    #[clap(long)]
    pub votes: Vec<u32>,
}

/// A subcommand for controlling the Voting Authority
#[derive(Clap, Debug)]
pub struct VotingAuthority {
    /// The voting authority subcommands
    #[clap(subcommand)]
    pub subcmd: VASubCommand,
}

#[derive(Clap, Debug)]
pub enum VASubCommand {
    #[clap(name = "setup")]
    SetupVote(SetupVote),
    #[clap(name = "store_question")]
    StoreQuestion(StoreQuestion),
    #[clap(name = "set_phase")]
    SetVotePhase(SetVotePhase),
}

/// A subcommand for setting up the vote
#[derive(Clap, Debug)]
pub struct SetupVote {
    /// The name of the vote
    #[clap(short, long)]
    pub vote: String,
    /// The question to store
    #[clap(short, long)]
    pub question: String,
}

/// A subcommand for setting up vote questions
#[derive(Clap, Debug)]
pub struct StoreQuestion {
    /// The name of the vote
    #[clap(short, long)]
    pub vote: String,
    /// The question to store
    #[clap(short, long)]
    pub question: String,
}

/// A subcommand for changing the vote phase
#[derive(Clap, Debug)]
pub struct SetVotePhase {
    /// The id of the vote to associate the question with
    #[clap(short, long)]
    pub vote: String,
    /// The  of the vote to create
    #[clap(short, long, possible_values = &["KeyGeneration", "Voting", "Tallying"])]
    pub phase: String,
}

/// A subcommand for controlling the Sealer
#[derive(Clap, Debug)]
pub struct Sealer {
    /// The sealer subcommands
    #[clap(subcommand)]
    pub subcmd: SealerSubCommand,
}

#[derive(Clap, Debug)]
pub enum SealerSubCommand {
    #[clap(name = "keygen")]
    KeyGeneration(KeyGeneration),
    #[clap(name = "decrypt")]
    PartialDecryption(PartialDecryption),
}

/// A subcommand for controlling the key generation
#[derive(Clap, Debug)]
pub struct KeyGeneration {
    /// The name of the sealer to use
    #[clap(short, long, required = true)]
    pub who: String,
}

/// A subcommand for controlling the partial decryption
#[derive(Clap, Debug)]
pub struct PartialDecryption {
    /// The name of the sealer to use
    #[clap(short, long, required = true)]
    pub who: String,
}
