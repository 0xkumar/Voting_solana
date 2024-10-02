use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    program::{invoke_signed},
    sysvar::Sysvar,
    sysvar::slot_history::ProgramError,
};

// Define the structure of our vote account
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VoteAccount {
    pub topic: String,
    pub yes_votes: u32,
    pub no_votes: u32,
}

// Define our instruction set
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum VoteInstruction {
    InitializeVote { topic: String },
    CastVote { vote: bool },
}

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = VoteInstruction::try_from_slice(instruction_data)?;

    match instruction {
        VoteInstruction::InitializeVote { topic } => {
            initialize_vote(program_id, accounts, topic)
        }
        VoteInstruction::CastVote { vote } => cast_vote(accounts, vote),
    }
}

pub fn initialize_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    topic: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let vote_account = next_account_info(account_info_iter)?;
    let user = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // Derive PDA
    let (pda, bump_seed) = Pubkey::find_program_address(
        &[b"vote_account", user.key.as_ref()],
        program_id,
    );

    // Ensure the derived address matches the given account
    if pda != *vote_account.key {
        return Err(ProgramError::InvalidAccountData.into());
    }

    // Calculate rent-exempt balance
    let rent = Rent::get()?;
    let space = 32 + 4 + 4; // Rough estimate of account size
    let lamports = rent.minimum_balance(space);

    // Create account 
    invoke_signed(
        &system_instruction::create_account(
            user.key,
            vote_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[user.clone(), vote_account.clone(), system_program.clone()],
        &[&[b"vote_account", user.key.as_ref(), &[bump_seed]]],
    )?;

    // Initialize vote account data
    let vote_account_data = VoteAccount {
        topic,
        yes_votes: 0,
        no_votes: 0,
    };
    //Serializing and putting the votedata in the data field of the vote_account PDA.
    vote_account_data.serialize(&mut &mut vote_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn cast_vote(accounts: &[AccountInfo], vote: bool) -> ProgramResult {

    let account_info_iter = &mut accounts.iter();
    let vote_account = next_account_info(account_info_iter)?;
    let user = next_account_info(account_info_iter)?;

    if !user.is_signer {
        return Err(ProgramError::MissingRequiredSignature.into());
    }

    let mut vote_data = VoteAccount::try_from_slice(&vote_account.data.borrow())?;

    if vote {
        vote_data.yes_votes += 1;
    } else {
        vote_data.no_votes += 1;
    }

    vote_data.serialize(&mut &mut vote_account.data.borrow_mut()[..])?;

    Ok(())
}
