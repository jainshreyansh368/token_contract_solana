
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::error::TokenCreateError::InvalidInstruction;
// use std::io::Read;


pub enum TokenCreateInstruction {
    CreateToken {
        /// decimal
        arg: u64,
    },

    MintToken {
        /// amount
        amount: u64,
    },

    AddToken {
        /// amount
        amount: u64,
    },

    Burn {
        amount: u64,
    },
}

impl TokenCreateInstruction {
    /// Unpacks a byte buffer into a [TokenCreateInstruction]
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::CreateToken {
                arg: Self::unpack_amount(rest)?,
            },
            1 => Self::MintToken {
                amount: Self::unpack_amount(rest)?,
            },
            2 => Self::AddToken {
                amount: Self::unpack_amount(rest)?,
            },
            3 => Self::Burn {
                amount: Self::unpack_amount(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
    
}