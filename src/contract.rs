use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, Storage,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, HandleAnswer, QueryAnswer};
use crate::state::{config, config_read, load, perform_transfer, may_load, save, State, Campaign};

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
        HandleMsg::Donate {name, amount} => try_donate(deps, env, name, amount),
        HandleMsg::Withdraw { name, amount } => try_withdraw(deps, env, name, amount),
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
    let sender_address = deps.api.canonical_address(&env.message.sender)?;
    //add new campaign name
    config(&mut deps.storage).update(|mut state| {
        let name = name.clone();
        state.campaign_names.push(name);
        Ok(state)
    })?;
    let stored_campaign = Campaign {
        owner: sender_address.to_string(),
        description: desc,
        amount: 0_u8,
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
    amount: u8,
) -> StdResult<HandleResponse> {
    let status: String;
    let new_value: u8;

    // read the campaign from storage
    let result: Option<Campaign> = may_load(&mut deps.storage, name.as_bytes()).ok().unwrap();
    match result {
        // set all response field values
        Some(mut stored_campaign) => {
            status = String::from("Campaign found!");
            perform_transfer(
                &mut deps.storage,
                &deps.api.canonical_address(&env.message.sender)?,
                &deps.api.canonical_address(&env.contract.address)?,
                amount.into()
            )?;
            new_value = stored_campaign.amount + amount;
            stored_campaign.amount = new_value;
            save(&mut deps.storage, name.as_bytes(), &stored_campaign)?;
        }
        // unless there's an error
        None => {
            new_value = 0;
            status = String::from("Campaign not found.");
        }
    }
    debug_print("donate successfully");
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Donate {
            status,
            new_value,
        })?),
    })
}

pub fn try_withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    amount: u8,
) -> StdResult<HandleResponse> {
    let status: String;
    let new_value: u8;

    // read the campaign from storage
    let result: Option<Campaign> = may_load(&mut deps.storage, name.as_bytes()).ok().unwrap();
    match result {
        // set all response field values
        Some(mut stored_campaign) => {
            let sender_address = deps.api.canonical_address(&env.message.sender)?;
            if stored_campaign.owner.eq(&sender_address.to_string()) {
                status = String::from("Campaign found! Withdraw permitted!");
                perform_transfer(
                    &mut deps.storage,
                    &deps.api.canonical_address(&env.contract.address)?,
                    &sender_address,
                    amount.into(),
                )?;
                new_value = stored_campaign.amount - amount;
                stored_campaign.amount = new_value;
                save(&mut deps.storage, name.as_bytes(), &stored_campaign)?;
            } else {
                status = String::from("Campaign found! Only campaign creator can withdraw!");
                new_value = 0;
            }
        }
        // unless there's an error
        None => {
            new_value = 0;
            status = String::from("Campaign not found.");
        }
    }
    debug_print("withdraw successfully");
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Donate {
            status,
            new_value,
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
