use crate::authentication::KeyPair;
use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use regex::Regex;
use bip39::{Language, Mnemonic};
use thiserror::Error;
use tracing::debug;

const MNEMONIC_LEN: usize = 24;
const MNEMONIC_CHECK_LEN: usize = 3;

#[derive(Debug, Error)]
pub enum IOError {
    #[error("Wrong answer in mnemonic check")]
    CheckMnemonicError,
    #[error("Error in user input: {0}")]
    InputError(#[from] std::io::Error),
    #[error("Error in KeyPair generation: {0}")]
    KeyPairError(#[from] ed25519_compact::Error),
    #[error("Mnemonic error: {0}")]
    MnemonicError(bip39::Error),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}

type Result<T> = std::result::Result<T, IOError>;

/// Helper function to get input from the user. Accept an optional [`Regex`] to
/// check the validity of the reply
pub fn get_user_input(request: &str, expected: Option<&Regex>) -> Result<String> {
    let mut response = String::new();

    loop {
        println!("{}", request);
        std::io::stdin().read_line(&mut response)?;
        response = response.trim().to_owned();

        match expected {
            Some(re) => {
                if re.is_match(response.as_str()) {
                    break;
                }
            },
            None => break,
        }

        response.clear();
        println!("Invalid reply, please answer again...");
    }

    Ok(response)
}

/// Generates a new [`KeyPair`] from a mnemonic provided by the user
pub fn generate_keypair() -> Result<KeyPair> {
    // Request mnemonic to the user
    let mnemonic_str = get_user_input(format!("Please provide a {} words mnemonic for your keypair:", MNEMONIC_LEN).as_str(), Some(&Regex::new(r"^([[:alpha:]]+\s){23}[[:alpha:]]+$")?))?;
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_str.as_str()).map_err(|e| {IOError::MnemonicError(e)})?;

    // FIXME: add unit tests for regex (proptest?)

    // Check if the user has correctly stored the mnemonic
    check_mnemonic(&mnemonic)?;

    let seed = mnemonic.to_seed_normalized("");
    Ok(KeyPair::try_from_seed(&seed)?)
}

/// Interactively check if the user has correctly stored the mnemonic phrase
fn check_mnemonic(mnemonic: &Mnemonic) -> Result<()> { //FIXME: improve prints
    let rng = rand::thread_rng();
    let uniform = Uniform::from(1..MNEMONIC_LEN);
    let random_indexes: Vec<usize> = uniform.sample_iter(rng).take(MNEMONIC_CHECK_LEN).collect(); //FIXME: this could get the same number more than once, check uniqueness of the index extracted

    println!("Mnemonic verification step");
    let mnemonic_slice: Vec<&'static str> = mnemonic.word_iter().collect();

    for i in random_indexes {
       let response = get_user_input(format!("Enter the word at index {} of your mnemonic:", i).as_str(), Some(&Regex::new(r"[[:alpha:]]+")?))?;
 
        if response != mnemonic_slice[i - 1] {
            debug!("Expected: {}, answer: {}", mnemonic_slice[i - 1], response);
            return Err(IOError::CheckMnemonicError);
        }
    }

    println!("Verification passed. Be sure to safely store your mnemonic phrase!");

    Ok(())
}