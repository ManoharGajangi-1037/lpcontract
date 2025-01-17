// use cosmwasm_std::Uint128;
// use cosmwasm_std::{Binary, CosmosMsg, CustomMsg, StdError, StdResult};
// use osmosis_std::types::osmosis::gamm::poolmodels::balancer::v1beta1::MsgCreateBalancerPool;
// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};

// /// Custom Osmosis message wrapper
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

// pub enum OsmosisMsg {
//     CreateBalancerPool(MsgCreateBalancerPool),
// }

// impl CustomMsg for OsmosisMsg {}

// impl From<OsmosisMsg> for CosmosMsg<OsmosisMsg> {
//     fn from(msg: OsmosisMsg) -> CosmosMsg<OsmosisMsg> {
//         CosmosMsg::Custom(msg)
//     }
// }

// impl From<MsgCreateBalancerPool> for OsmosisMsg {
//     fn from(msg: MsgCreateBalancerPool) -> OsmosisMsg {
//         OsmosisMsg::CreateBalancerPool(msg)
//     }
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct InstantiateMsg {}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum ExecuteMsg {
//     CreatePool {
//         token_a: Token,
//         token_b: Token,
//         lp_owner: String,
//     },
//     WithdrawLockedLpShares {},
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct Token {
//     pub denom: String,
//     pub amount: Uint128,
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub enum QueryMsg {}


use cosmwasm_std::Uint128;
use cosmwasm_std::{Binary, CosmosMsg, CustomMsg, StdError, StdResult};
use osmosis_std::types::osmosis::gamm::poolmodels::balancer::v1beta1::MsgCreateBalancerPool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Custom Osmosis message wrapper
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum OsmosisMsg {
    CreateBalancerPool(MsgCreateBalancerPool),
}

impl CustomMsg for OsmosisMsg {}

impl From<OsmosisMsg> for CosmosMsg<OsmosisMsg> {
    fn from(msg: OsmosisMsg) -> CosmosMsg<OsmosisMsg> {
        CosmosMsg::Custom(msg)
    }
}

impl From<MsgCreateBalancerPool> for OsmosisMsg {
    fn from(msg: MsgCreateBalancerPool) -> OsmosisMsg {
        OsmosisMsg::CreateBalancerPool(msg)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcCreatePoolMsg {
    pub token_a: Token,
    pub token_b: Token,
    pub lp_owner: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreatePool {
        token_a: Token,
        token_b: Token,
        lp_owner: String,
    },
    WithdrawLockedLpShares {},
    // Add IBC receive message handler
}

// New IBC Hook message type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IbcHookMsg {
    CreatePool {
        token_a: Token,
        token_b: Token,
        lp_owner: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub denom: String,
    pub amount: Uint128,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetChannelInfo {},
    GetCounter {},
}



// Add IBC Lifecycle messages for handling acknowledgments
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    IBCLifecycleComplete(IBCLifecycleComplete),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IBCLifecycleComplete {
    IBCAck {
        channel: String,
        sequence: u64,
        ack: String,
        success: bool,
    },
    IBCTimeout {
        channel: String,
        sequence: u64,
    },
}

// Response type for async IBC operations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IbcReceiveResponse {
    pub acknowledgement: Binary,
    pub messages: Vec<CosmosMsg<OsmosisMsg>>,
}