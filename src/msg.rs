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
        name: String
    },
    Withdraw {
        name: String,
        amount: u32
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
        status_value: String,
    },
    /// Return a status message and campaign amount after withdraw
    Withdraw {
        status: String,
        status_value: String,
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
        amount: String,
    }
}
