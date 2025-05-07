use anchor_lang::prelude::*;
use crate::errors::PerpError;


#[account]
pub struct VolatilityStats {
    pub authority: Pubkey,
    pub last_price: u64,            
    pub mean: f64,                  
    pub m2: f64,                    
    pub count: u64,                
    pub annualized_volatility: f64, 
}

impl VolatilityStats {
    pub const SIZE: usize = 8 +  
        32 +  
        8 +  
        8 +  
        8 +  
        8 +  
        8;  

    pub fn load_from_account_info(account_info: &AccountInfo) -> Result<f64> {
        let data = account_info.try_borrow_data()?;
        
        if data.len() < Self::SIZE {
            return Err(PerpError::OracleStale.into());
        }
        
        let start_index = 8 + 32 + 8 + 8 + 8 + 8; 
        let annualized_volatility_bytes = &data[start_index..start_index + 8];
        let annualized_volatility = f64::from_le_bytes(annualized_volatility_bytes.try_into().unwrap());
        
        Ok(annualized_volatility)
    }
}