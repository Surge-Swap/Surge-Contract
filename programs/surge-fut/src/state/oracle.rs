use anchor_lang::prelude::*;

#[derive(Clone, AnchorDeserialize, AnchorSerialize)]
pub struct VolatilityStats {
    pub authority: Pubkey,
    pub last_price: u64,
    pub mean: f64,
    pub m2: f64,
    pub count: u64,
    pub annualized_volatility: f64,
}

impl VolatilityStats {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 8 + 8;

    pub fn load_from_account_info(account_info: &AccountInfo) -> Result<f64> {
        let data = account_info.try_borrow_data()?;
        
        if data.len() < Self::SIZE {
            return Err(crate::errors::ContractError::InvalidOracleData.into());
        }
        
        // Skip the 8-byte discriminator if present
        let mut offset = 8;
        
        // Skip authority (Pubkey - 32 bytes)
        offset += 32;
        
        // Skip last_price (u64 - 8 bytes)
        offset += 8;
        
        // Skip mean (f64 - 8 bytes)
        offset += 8;
        
        // Skip m2 (f64 - 8 bytes)
        offset += 8;
        
        // Skip count (u64 - 8 bytes)
        offset += 8;
        
        // Read annualized_volatility (f64 - 8 bytes)
        let annualized_volatility_bytes = &data[offset..offset + 8];
        let annualized_volatility = f64::from_le_bytes(annualized_volatility_bytes.try_into().unwrap());
        
        Ok(annualized_volatility)
    }
}