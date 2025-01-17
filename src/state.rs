
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
pub const TOTAL_STAKED: Item<Uint128> = Item::new("total_staked");
pub const STAKED_BALANCES: Map<Addr, Uint128> = Map::new("staked_balances");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockedShares {
    pub pool_id: Uint128,
    pub amount: Uint128,
    pub unlock_time: u64,
}

pub const LOCK_DURATION: Item<u64> = Item::new("lock_duration");
use cosmwasm_std::IbcChannel;
pub const COUNTER: Item<u64> = Item::new("counter");
pub const CHANNEL_INFO: Item<IbcChannel> = Item::new("channel_info");
// Store locked LP shares in the contract state
pub static LOCKED_LP_SHARES: Map<&Addr, LockedShares> = Map::new("locked_lp_shares");
pub const SENDER_ADDRESS: Item<Addr> = Item::new("sender_address");
