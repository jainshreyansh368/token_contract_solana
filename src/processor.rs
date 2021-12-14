use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    system_instruction::{create_account, transfer},
};

use spl_token::{
    state::{Account as TokenAccount, Mint},
    instruction::initialize_mint,
};

use crate::{error::TokenCreateError, instruction::TokenCreateInstruction, state::TokenData };
pub struct Processor;

impl Processor{
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = TokenCreateInstruction::unpack(instruction_data)?;

        // Escrow instructions
        match instruction {
            TokenCreateInstruction::CreateToken { arg } => {
                msg!("Instruction: Creating Token");
                Self::process_init_token_state(accounts, arg, program_id)
            }
            TokenCreateInstruction::MintToken { amount } => {
                msg!("Instruction: Minting fresh tokens");
                Self::process_mint_token(accounts, amount, program_id)
            }
            TokenCreateInstruction::AddToken { amount } => {
                msg!("Instruction: Minting more tokens");
                Self::process_add_mint(accounts, amount, program_id)
            }
            TokenCreateInstruction::Burn { amount } => {
                msg!("Instruction: Minting more tokens");
                Self::process_burn(accounts, amount, program_id)
            }
        }
    }

    pub fn process_init_token_state(
        accounts: &[AccountInfo],
        arg: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        // initializer is signer validation check
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let token_state_account = next_account_info(account_info_iter)?;
        let mint_key = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

            invoke(
                &create_account(
                    initializer.key, 
                    token_state_account.key, 
                    Rent::default().
                    minimum_balance(
                    TokenData::LEN 
                    ),
                    TokenData::LEN as u64, 
                    &program_id,   
                ),
                &[
                    initializer.clone(),
                    token_state_account.clone(),
                    system_program.clone(),
                ],
            )?;

        let mut token_info = TokenData::unpack_unchecked(&token_state_account.try_borrow_data()?)?;

        if token_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        // set the state for TokenData 
        token_info.is_initialized = true;
        token_info.decimal = arg;
        token_info.mint = *mint_key.key;
        token_info.initializer = *initializer.key;
        TokenData::pack(token_info, &mut token_state_account.try_borrow_mut_data()?)?;
        Ok(())
    }

    pub fn process_mint_token(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        // user is signer validation check
        if !user.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let token_account = next_account_info(account_info_iter)?;
        let token_state_account = next_account_info(account_info_iter)?;

        let token_info = TokenData::unpack(&token_state_account.try_borrow_data()?)?;

        let mint_key = token_info.mint.clone();
        let decimal = token_info.decimal.clone();
        if *user.key != token_info.initializer {
            return Err(TokenCreateError::InvalidInstruction.into());
        }

        let token_program = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

            invoke(
                &create_account(
                    user.key, 
                    token_state_account.key, 
                    Rent::default().
                    minimum_balance(
                    TokenData::LEN 
                    ),
                    TokenData::LEN as u64, 
                    &spl_token::id(),   
                ),
                &[
                    user.clone(),
                    token_state_account.clone(),
                    system_program.clone(),
                ],
            )?;

        let initmint = spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint_key,
                user.key,
                None,
                decimal as u8,
            )?;

        invoke(
            &initmint,
            &[
                user.clone(),
                token_program.clone(),
            ],
        )?;

        let mintTo = spl_token::instruction::mint_to(
            &spl_token::id(), 
            &mint_key, 
            token_account.key, 
            user.key, 
            &[], 
            amount,
        )?;
        invoke(
            &mintTo,
            &[
                token_account.clone(),
                user.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn process_add_mint(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        // initializer is signer validation check
        if !user.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let token_account = next_account_info(account_info_iter)?;
        let token_state_account = next_account_info(account_info_iter)?;

        let token_info = TokenData::unpack(&token_state_account.try_borrow_data()?)?;
        let mint_key = token_info.mint.clone();
        let decimal = token_info.decimal.clone();
        if *user.key != token_info.initializer {
            return Err(TokenCreateError::InvalidInstruction.into());
        }

        let token_program = next_account_info(account_info_iter)?;

        let mintTo = spl_token::instruction::mint_to(
            &spl_token::id(), 
            &mint_key, 
            token_account.key, 
            user.key, 
            &[], 
            amount,
        )?;
        invoke(
            &mintTo,
            &[
                token_account.clone(),
                user.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }


    pub fn process_burn(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        // initializer is signer validation check
        if !user.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let token_account = next_account_info(account_info_iter)?;
        let token_state_account = next_account_info(account_info_iter)?;

        let token_info = TokenData::unpack(&token_state_account.try_borrow_data()?)?;
        let mint_key = token_info.mint.clone();
        let decimal = token_info.decimal.clone();
        if *user.key != token_info.initializer {
            return Err(TokenCreateError::InvalidInstruction.into());
        }

        let token_program = next_account_info(account_info_iter)?;

        let mintTo = spl_token::instruction::burn(
            &spl_token::id(),  
            token_account.key,
            &mint_key, 
            user.key, 
            &[], 
            amount,
        )?;
        invoke(
            &mintTo,
            &[
                token_account.clone(),
                user.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }
    
       
}