use crate::msg::{ExecuteMsg, IbcCreatePoolMsg, InstantiateMsg, Token};
use crate::state::{
    LockedShares, CHANNEL_INFO, COUNTER, LOCKED_LP_SHARES, LOCK_DURATION, SENDER_ADDRESS,
};
use crate::ContractError;
use cosmwasm_std::{
    entry_point, to_binary, to_json_binary, BankMsg, IbcAcknowledgement, IbcChannel,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcDestinationCallbackMsg,
    IbcPacketReceiveMsg, Reply, StdAck, SubMsg, SubMsgResponse, SubMsgResult,
};
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use cosmwasm_std::{Coin, IbcReceiveResponse};
use osmosis_std::types::cosmos::base::v1beta1::Coin as OsmosisCoin;
use osmosis_std::types::osmosis::gamm::poolmodels::balancer::v1beta1::{
    MsgCreateBalancerPool, MsgCreateBalancerPoolResponse,
};
use osmosis_std::types::osmosis::gamm::v1beta1::{PoolAsset, PoolParams};

use cosmwasm_std::IbcBasicResponse;
const CREATE_POOL_REPLY_ID: u64 = 1;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    let lock_duration = 600u64;

    // Save the lock duration using `cw-storage-plus`
    LOCK_DURATION.save(deps.storage, &lock_duration)?;
    let _ = COUNTER.save(deps.storage, &0u64);
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::CreatePool {
            token_a,
            token_b,
            lp_owner,
        } => create_pool(deps, env, info, token_a, token_b, lp_owner),
        ExecuteMsg::WithdrawLockedLpShares {} => withdraw_locked_lp_shares(deps, env, info),
    }
}

use cosmwasm_std::{BalanceResponse, BankQuery, QueryRequest};
use serde::Deserialize;
use serde_json::to_vec;

fn query_funds(deps: &DepsMut, denom: &str, contract_address: &str) -> StdResult<Uint128> {
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address: contract_address.to_string(),
        denom: denom.to_string(),
    }))?;

    Ok(balance.amount.amount)
}

#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let mut counter = COUNTER.may_load(deps.storage)?.unwrap_or(0);
    counter += 5;
    COUNTER.save(deps.storage, &counter)?;

    let wasm_msg: WasmMessage = serde_json::from_slice(&msg.packet.data)
        .map_err(|_| StdError::generic_err("Failed to parse packet data"))?;

    // Validate WasmMessage structure
    if let Some(contract_msg) = wasm_msg.wasm {
        if contract_msg.contract.is_empty() || contract_msg.msg.is_empty() {
            return Err(StdError::generic_err("Invalid contract message structure"));
        }

        let execute_msg: Result<ExecuteMsg, _> = serde_json::from_str(&contract_msg.msg);
        if let Ok(ExecuteMsg::CreatePool {
            token_a,
            token_b,
            lp_owner,
        }) = execute_msg
        {
            // Query the transferred funds
            let contract_address = _env.contract.address.to_string();
            let amount_a = query_funds(&deps, &token_a.denom, &contract_address)?;
            let amount_b = query_funds(&deps, &token_b.denom, &contract_address)?;

            // Create MessageInfo with the contract as sender
            let info = MessageInfo {
                sender: _env.contract.address.clone(),
                funds: vec![
                    Coin {
                        denom: token_a.denom.clone(),
                        amount: amount_a,
                    },
                    Coin {
                        denom: token_b.denom.clone(),
                        amount: amount_b,
                    },
                ],
            };

            // Execute the pool creation
            let response = create_pool(deps, _env, info, token_a, token_b, lp_owner)?;

            // Acknowledge success
            let ack = IbcAcknowledgement::new(to_binary(&StdAck::success(b"success"))?);
            return Ok(IbcReceiveResponse::new(b"transaction")
                .add_attribute("action", "ibc_packet_receive")
                .add_attributes(response.attributes));
        }
    }

    Ok(IbcReceiveResponse::new(b"success").add_attribute("action", "ibc_channel_connect"))
}

#[entry_point]
pub fn ibc_destination_callback(
    deps: DepsMut,
    env: Env,
    msg: IbcDestinationCallbackMsg,
) -> StdResult<IbcBasicResponse> {
    let mut counter = COUNTER.may_load(deps.storage)?.unwrap_or(0);
    counter += 1;
    COUNTER.save(deps.storage, &counter)?;
    deps.api
        .debug(&format!("Received packet data: {:?}", msg.packet.data));
    // Parse packet data

    Ok(IbcBasicResponse::new())
}

// Add these structs to handle the message
#[derive(Deserialize)]
struct WasmMessage {
    wasm: Option<ContractMessage>,
}

#[derive(Deserialize)]
struct ContractMessage {
    contract: String,
    msg: String,
}

pub fn create_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_a: Token,
    token_b: Token,
    lp_owner: String,
) -> StdResult<Response> {
    // Ensure 1:1 token ratio
    if token_a.amount != token_b.amount {
        return Err(StdError::generic_err("Token ratios must be 1:1"));
    }

    // Validate the LP owner address
    let owner_addr = deps.api.addr_validate(&lp_owner)?;

    // if owner_addr != info.sender {
    //     return Err(StdError::generic_err(
    //         "LP owner must be the sender of the transaction",
    //     ));
    // }

    // Create pool assets
    let pool_assets = vec![
        PoolAsset {
            token: Some(OsmosisCoin {
                denom: token_a.denom.clone(),
                amount: token_a.amount.into(),
            }),
            weight: "500000".to_string(),
        },
        PoolAsset {
            token: Some(OsmosisCoin {
                denom: token_b.denom.clone(),
                amount: token_b.amount.into(),
            }),
            weight: "500000".to_string(),
        },
    ];

    // Set pool parameters
    let pool_params = Some(PoolParams {
        swap_fee: "3000000000000000".to_string(), // 0.003 swap fee
        exit_fee: "0".to_string(),
        smooth_weight_change_params: None,
    });

    // Construct the MsgCreateBalancerPool message
    let create_pool_msg = MsgCreateBalancerPool {
        sender: env.contract.address.to_string(),
        pool_params,
        pool_assets,
        future_pool_governor: owner_addr.to_string(),
    };

    // Save the sender address for reply handling
    SENDER_ADDRESS.save(deps.storage, &info.sender)?;

    // Send the pool creation message
    let sub_msg = SubMsg::reply_on_success(create_pool_msg, CREATE_POOL_REPLY_ID);

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "create_pool")
        .add_attribute("lp_owner", lp_owner))
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != CREATE_POOL_REPLY_ID {
        return Ok(Response::new());
    }

    // Check if the SubMsgResult is successful
    if let SubMsgResult::Ok(SubMsgResponse { data, .. }) = msg.result {
        // Extract and parse the response data
        let response_data = data.ok_or_else(|| {
            ContractError::Std(StdError::generic_err("No response data available"))
        })?;

        let res: MsgCreateBalancerPoolResponse =
            response_data.try_into().map_err(ContractError::Std)?;

        // Retrieve and clean up the sender's address from storage
        let sender = SENDER_ADDRESS.load(deps.storage)?;
        SENDER_ADDRESS.remove(deps.storage);

        // Load lock duration and calculate unlock time
        let lock_duration: u64 = LOCK_DURATION.load(deps.storage)?;
        let unlock_time = env.block.time.seconds() + lock_duration;

        // Save the locked LP shares
        LOCKED_LP_SHARES.save(
            deps.storage,
            &sender,
            &LockedShares {
                pool_id: Uint128::from(res.pool_id),
                amount: Uint128::new(1000000), // Replace with dynamic value if needed
                unlock_time,
            },
        )?;

        // Return the response with attributes
        return Ok(Response::new()
            .add_attribute("action", "create_pool")
            .add_attribute("pool_id", res.pool_id.to_string())
            .add_attribute("unlock_time", unlock_time.to_string()));
    }

    // If the SubMsgResult is not Ok, return an empty response
    Ok(Response::new())
}

pub fn withdraw_locked_lp_shares(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    // Load locked LP shares for the sender
    let locked_shares = LOCKED_LP_SHARES.may_load(deps.storage, &info.sender)?;

    if let Some(shares) = locked_shares {
        // Check if lock period has expired
        if env.block.time.seconds() < shares.unlock_time {
            let remaining_time = shares.unlock_time - env.block.time.seconds();
            return Err(StdError::generic_err(format!(
                "LP shares are still locked. Please wait {} more seconds.",
                remaining_time
            )));
        }

        // Remove locked shares from storage
        LOCKED_LP_SHARES.remove(deps.storage, &info.sender);

        // Send LP shares to the user
        let send_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: format!("gamm/pool/{}", shares.pool_id),
                amount: shares.amount,
            }],
        };

        // Return response
        Ok(Response::new()
            .add_message(send_msg)
            .add_attribute("method", "withdraw_locked_lp_shares")
            .add_attribute("amount_withdrawn", shares.amount.to_string()))
    } else {
        Err(StdError::generic_err("No locked LP shares found"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelOpenMsg,
) -> StdResult<IbcChannelOpenResponse> {
    Ok(None)
}
#[entry_point]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();

    // Save the IBC channel information
    CHANNEL_INFO.save(deps.storage, &channel)?;

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_channel_connect")
        .add_attribute("channel_id", channel.endpoint.channel_id.to_string())
        .add_attribute("port_id", channel.endpoint.port_id.to_string()))
}

#[entry_point]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: cosmwasm_std::IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();

    // Remove saved channel info
    CHANNEL_INFO.remove(deps.storage);

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_channel_close")
        .add_attribute("channel_id", channel.endpoint.channel_id.to_string()))
}

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: cosmwasm_std::IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    let ack: Result<Binary, _> = serde_json::from_slice(&msg.acknowledgement.data);

    match ack {
        Ok(_) => Ok(IbcBasicResponse::new().add_attribute("action", "acknowledged")),
        Err(_) => Ok(IbcBasicResponse::new().add_attribute("action", "ack_error")),
    }
}

#[entry_point]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: cosmwasm_std::IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    let packet = msg.packet;
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "timeout")
        .add_attribute("sequence", packet.sequence.to_string()))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: crate::msg::QueryMsg) -> StdResult<Binary> {
    match msg {
        crate::msg::QueryMsg::GetChannelInfo {} => to_json_binary(&query_channel_info(deps)?),
        crate::msg::QueryMsg::GetCounter {} => to_json_binary(&query_counter(deps)?),
    }
}

fn query_counter(deps: Deps) -> StdResult<u64> {
    let counter = COUNTER.may_load(deps.storage)?.unwrap_or(0);
    Ok(counter)
}
fn query_channel_info(deps: Deps) -> StdResult<IbcChannel> {
    let channel_info = CHANNEL_INFO.load(deps.storage)?;
    Ok(channel_info)
}
