use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};

use near_sdk::log;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};

const PUZZLE_NUMBER: u8 = 1;

// 5 Ⓝ in yoctoNEAR

const PRIZE_AMOUNT: u128 = 5_000_000_000_000_000_000_000_000;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct CrossWord {
    owner_id: AccountId,
    puzzles: LookupMap<String, Puzzle>,
    unsolved_puzzles: UnorderedSet<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Puzzle {
    status: PuzzleStatus,

    answer: Vec<Answer>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Answer {
    num: u8,
    start: CoordinatePair,
    direction: AnswerDirection,
    lenght: u8,
    clue: String,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CoordinatePair {
    x: u8,
    y: u8,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonPuzzle {
    /// The human-readable (not in bytes) hash of the solution
    solution_hash: String, // ⟵ this field is not contained in the Puzzle struct
    status: PuzzleStatus,
    answer: Vec<Answer>,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum AnswerDirection {
    Across,
    Down,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum PuzzleStatus {
    Unsolved,
    Solved { memo: String },
}

#[near_bindgen]
impl CrossWord {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            puzzles: LookupMap::new(b"c"),
            unsolved_puzzles: UnorderedSet::new(b"u"),
        }
    }

    pub fn new_puzzle(&mut self, solution_hash: String, answers: Vec<Answer>) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only the owner may call this method"
        );
        let existing = self.puzzles.insert(
            &solution_hash,
            &Puzzle {
                status: PuzzleStatus::Unsolved,
                answer: answers,
            },
        );

        assert!(existing.is_none(), "Puzzle with that key already exists");
        self.unsolved_puzzles.insert(&solution_hash);
    }

    pub fn get_pazzle_number(&self) -> u8 {
        PUZZLE_NUMBER
    }

    // pub fn set_solution(&mut self, solution: String) {
    //     self.crossword_solution = solution;
    // }

    // pub fn get_solution(&self) -> String {
    //     self.crossword_solution.clone()
    // }

    pub fn submit_solution(&mut self, solution: String, memo: String) {
        let hashed_input = env::sha256(solution.as_bytes());
        let hashed_input_hex = hex::encode(&hashed_input);

        let answer_pk = env::signer_account_pk();

        let mut puzzle = self
            .puzzles
            .get(&hashed_input_hex)
            .expect("ERR_NOT_CORRECT_ANSWER");

        puzzle.status = match puzzle.status {
            PuzzleStatus::Unsolved => PuzzleStatus::Solved { memo: memo.clone() },
            _ => {
                env::panic_str("ERR_PUZZLE_SOLVED");
            }
        };
        self.puzzles.insert(&hashed_input_hex, &puzzle);

        self.unsolved_puzzles.remove(&hashed_input_hex);

        log!(
            "Puzzle with solution hash {} solved, with memo: {}",
            hashed_input_hex,
            memo
        );

        // Delete old function call key
        Promise::new(env::current_account_id()).delete_key(answer_pk);
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();

        builder.predecessor_account_id(predecessor);
        builder
    }

    // #[test]
    // fn check_guess_solution() {
    //     let alice = AccountId::new_unchecked("alice.testnet".to_string());

    //     let context = get_context(alice);
    //     testing_env!(context.build());

    //     let mut contract = Contract::new(
    //         "fb699b30bf500f810f8cd83816a85c172dd760208b929d0dd25703578bfe669b".to_string(),
    //     );

    //     let mut _guess_result = contract.guess_solution("wrong answer here".to_string());
    //     assert_eq!(get_logs(), ["Try again."], "Expected a failure log.");
    // }

    #[test]
    fn debug_get_hash() {
        testing_env!(VMContextBuilder::new().build());

        let debug_solution = "georzynskyi";
        let debug_hash_bytes = env::sha256(debug_solution.as_bytes());
        let debug_hash_string = hex::encode(debug_hash_bytes);
        println!("Let's debug: {:?}", debug_hash_string);
    }
}
