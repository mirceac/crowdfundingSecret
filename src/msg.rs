use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Create {
        name: String,
        description: String
    },
    Donate {
        name: String,
        amount: u8
    },
    Withdraw {
        name: String,
        amount: u8
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current campaign count
    Campaigns {},
    Campaign {
        name : String
    },
}

/// Responses from handle function
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    // add HandleMsg response types here
    Create {
      status: String
    },
    /// Return a status message and campaign amount after donation
    Donate {
        status: String,
        new_value: u8,
    },
    /// Return a status message and campaign amount after withdraw
    Withdraw {
        status: String,
        new_value: u8,
    }
}

/// Responses from query function
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    /// Return campain names
    Campaigns {
        names: Vec<String>,
    },
    Campaign {
        owner: String,
        description: String,
        amount: u8,
    }
}
