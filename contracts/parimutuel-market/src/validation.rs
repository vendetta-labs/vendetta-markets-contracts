use crate::error::ContractError;

pub fn validate_fee_bps(fee_bps: u64) -> Result<(), ContractError> {
    if fee_bps > 1_000 {
        return Err(ContractError::InvalidFeeBps(fee_bps));
    }

    Ok(())
}
