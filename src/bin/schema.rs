use cosmwasm_schema::write_api;

use osmosis_liquidity_pool::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        // query: QueryMsg,
    }
}
