use cosmwasm_std::Uint128;

pub fn calculate_parimutuel_winnings(
    total_bets: u128,
    total_team_bets: u128,
    total_bet: u128,
) -> u128 {
    if total_bet == 0 || total_team_bets == 0 || total_bets == 0 {
        return 0;
    }

    Uint128::from(total_bets)
        .multiply_ratio(total_bet, total_team_bets)
        .u128()
}
