use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TokenData {
    pub is_initialized: bool,
    pub decimal: u64,
    pub mint: Pubkey,
    pub initializer: Pubkey,
}

impl Sealed for TokenData {}
impl IsInitialized for TokenData {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for TokenData {
    const LEN: usize = 73;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, TokenData::LEN];
        let (
            is_initialized,
            decimal,
            mint,
            initializer,
        ) = array_refs![src, 1, 8, 32, 32];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(TokenData {
            is_initialized,
            decimal: u64::from_le_bytes(*decimal),
            mint: Pubkey::new_from_array(*mint), 
            initializer: Pubkey::new_from_array(*initializer),    
   
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, TokenData::LEN];
        let (
            is_initialized_dst,
            decimal_dst,
            mint_dst,
            initializer_dst,
        ) = mut_array_refs![dst, 1, 8, 32, 32];
        let TokenData {
            is_initialized,
            decimal,
            mint,
            initializer,
        } = self;
        is_initialized_dst[0] = *is_initialized as u8;
        *decimal_dst = decimal.to_le_bytes();
        mint_dst.copy_from_slice(mint.as_ref());
        initializer_dst.copy_from_slice(initializer.as_ref());

    }
}
