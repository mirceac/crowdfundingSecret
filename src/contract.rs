use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    Coin, BankMsg, CosmosMsg, StdResult, StdError, Storage,Uint128,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, HandleAnswer, QueryAnswer};
use crate::state::{config, config_read, load, may_load, save, State, Campaign};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        campaign_names: vec![],
    };

    config(&mut deps.storage).save(&state)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Create {name, description} => try_create(deps, env, name, description),
        HandleMsg::Donate {name} => try_donate(deps, env, name),
        HandleMsg::Withdraw { name, amount } => try_withdraw(deps, env, name, amount.into()),
    }
}

pub fn try_create<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    desc: String,
) -> StdResult<HandleResponse> {
    let status: String;
    // get the canonical address of sender
    let sender_address = env.message.sender;
    //add new campaign name
    config(&mut deps.storage).update(|mut state| {
        let name = name.clone();
        state.campaign_names.push(name);
        Ok(state)
    })?;
    let stored_campaign = Campaign {
        owner: sender_address.to_string(),
        description: desc,
        amount: "0".to_string(),
    };

    save(&mut deps.storage, name.as_bytes(), &stored_campaign)?;
    status = String::from(format!("Campaign {} created" , name));
    debug_print("create successfully");
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Create {
            status,
        })?),
    })
}

pub fn try_donate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
) -> StdResult<HandleResponse> {
    let status: String;
    let status_value: String;
    let mut new_value: u128;

    // read the campaign from storage
    let result: Option<Campaign> = may_load(&mut deps.storage, name.as_bytes()).ok().unwrap();
    match result {
        // set all response field values
        Some(mut stored_campaign) => {
            if env.message.sent_funds.len() != 1
                || env.message.sent_funds[0].amount
                < Uint128(1_000 /* 1mn uscrt = 1 SCRT */)
                || env.message.sent_funds[0].denom != String::from("uscrt") {
                return Err(StdError::generic_err( "Please donate at least 0,001 SCRT."));
            } else {
                new_value = stored_campaign.amount.parse::<u128>().unwrap();
                new_value += env.message.sent_funds[0].amount.u128();
                stored_campaign.amount = new_value.to_string();
                save(&mut deps.storage, name.as_bytes(), &stored_campaign)?;
                status_value = stored_campaign.amount;
                status = String::from("Campaign found! Donation sent!");
            }
        }
        // unless there's an error
        None => {
            return Err(StdError::generic_err("Campaign not found!"));
        }
    }
    debug_print("donate successfully");
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Donate {
            status,
            status_value,
        })?),
    })
}

pub fn try_withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    amount: u128,
) -> StdResult<HandleResponse> {
    let status: String;
    let new_value: u128;
    let status_value: String;
    // read the campaign from storage
    let result: Option<Campaign> = may_load(&mut deps.storage, name.as_bytes()).ok().unwrap();
    match result {
        // set all response field values
        Some(mut stored_campaign) => {
            if stored_campaign.owner.eq(&env.message.sender.to_string()) {
                status = String::from("Campaign found! Withdraw permitted and executed!");
                new_value = stored_campaign.amount.parse::<u128>().unwrap() - amount;
                stored_campaign.amount = new_value.to_string();
                save(&mut deps.storage, name.as_bytes(), &stored_campaign)?;
                status_value = stored_campaign.amount;
            } else {
                return Err(StdError::generic_err("Campaign found! Only campaign creator can withdraw!"));
            }
        }
        // unless there's an error
        None => {
            return Err(StdError::generic_err("Campaign not found."));
        }
    }
    debug_print("withdraw successfully");
    Ok(HandleResponse {
        messages: vec![CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address,
                to_address: env.message.sender,
                amount: vec![Coin {
                    denom: "uscrt".into(),
                    amount: Uint128(amount.into()),
                }]
            })],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Withdraw {
            status,
            status_value,
        })?),
    })
}


pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Campaigns {} => query_campaigns(deps),
        QueryMsg::Campaign {name} => query_campaign(deps, name),
    }
}

fn query_campaigns<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let state = config_read(&deps.storage).load()?;
    to_binary(&QueryAnswer::Campaigns{ names: state.campaign_names })
}

fn query_campaign<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, name: String) -> StdResult<Binary> {
    let config: Campaign = load(&deps.storage, name.as_bytes())?;
    to_binary(&QueryAnswer::Campaign{ owner: config.owner, description: config.description, amount: config.amount })
}
