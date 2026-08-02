//! Rule data types matching shiftRulesData.json.

use serde::{Deserialize, Serialize};

/// Difficulty level for a rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Medium,
    Hard,
    Expert,
    Nightmare,
}

/// Category of rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum RuleType {
    #[default]
    Basic,
    Conditional,
    Conflicting,
    Hidden,
    Weather,
}

/// Whether an action is forbidden or required
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Forbidden,
    Required,
}

/// Type of consequence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsequenceType {
    Death,
    Survival,
    Reputation,
    Money,
    Fuel,
    Time,
    Item,
    StoryUnlock,
}

/// A consequence that can occur from following or breaking rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consequence {
    #[serde(rename = "type")]
    pub consequence_type: ConsequenceType,
    pub value: i32,
    pub description: String,
    /// An unauthored probability means the consequence always fires. The
    /// bare `#[serde(default)]` read 0.0 here — a consequence that never
    /// fires — silently contradicting the struct's own `Default`.
    #[serde(default = "default_probability")]
    pub probability: f32,
    /// Catalog name of the thing an `Item` consequence hands over. Unnamed
    /// item consequences fall back to a crumpled note.
    #[serde(default)]
    pub item: Option<String>,
}

fn default_probability() -> f32 {
    1.0
}

impl Default for Consequence {
    fn default() -> Self {
        Self {
            consequence_type: ConsequenceType::Survival,
            value: 0,
            description: String::new(),
            probability: 1.0,
            item: None,
        }
    }
}

/// A shift rule that must be followed or broken strategically
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Rule {
    pub id: u32,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub difficulty: Difficulty,
    #[serde(rename = "type", default)]
    pub rule_type: RuleType,
    #[serde(default)]
    pub visible: bool,
    #[serde(rename = "actionKey")]
    pub action_key: Option<String>,
    #[serde(rename = "actionType")]
    pub action_type: Option<ActionType>,
    #[serde(rename = "relatedGuidelineId")]
    pub related_guideline_id: Option<u32>,
    #[serde(rename = "defaultOutcome")]
    pub default_outcome: Option<String>,
    // A rule carries no exceptions of its own. None of the twenty-eight
    // authors any, nothing read the field, and it was typed as raw
    // `serde_json::Value` because there was no shape to give it. A rule
    // reaches its exceptions through `related_guideline_id` -- the guideline
    // that owns them.
    #[serde(rename = "followConsequences", default)]
    pub follow_consequences: Vec<Consequence>,
    #[serde(rename = "breakConsequences", default)]
    pub break_consequences: Vec<Consequence>,
    #[serde(rename = "exceptionRewards", default)]
    pub exception_rewards: Vec<Consequence>,
    #[serde(rename = "exceptionNeedAdjustment")]
    pub exception_need_adjustment: Option<i32>,
    #[serde(rename = "followNeedAdjustment")]
    pub follow_need_adjustment: Option<i32>,
    #[serde(rename = "breakNeedAdjustment")]
    pub break_need_adjustment: Option<i32>,
    #[serde(rename = "conflictsWith", default)]
    pub conflicts_with: Vec<u32>,
    pub trigger: Option<String>,
    #[serde(rename = "violationMessage")]
    pub violation_message: Option<String>,
}

impl Rule {
    /// Check if this rule forbids a specific action
    pub fn forbids_action(&self, action: &str) -> bool {
        if let (Some(key), Some(ActionType::Forbidden)) = (&self.action_key, &self.action_type) {
            key == action
        } else {
            false
        }
    }

    /// Get the violation message or a default one
    pub fn get_violation_message(&self) -> &str {
        self.violation_message
            .as_deref()
            .unwrap_or("You violated a rule. The consequences are upon you...")
    }
}

/// A guideline exception condition
// The JSON also authors a `description` per condition; it is a note for the
// data author, read by nothing, and not deserialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionCondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub value: serde_json::Value,
    pub operator: Option<String>,
}

/// A guideline exception that inverts normal rule behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidelineException {
    pub id: String,
    #[serde(rename = "passengerIds", default)]
    pub passenger_ids: Vec<u32>,
    #[serde(rename = "passengerTypes", default)]
    pub passenger_types: Vec<String>,
    #[serde(default)]
    pub conditions: Vec<ExceptionCondition>,
    #[serde(default)]
    pub tells: Vec<super::passenger::PassengerTell>,
    #[serde(rename = "breakingSafer", default)]
    pub breaking_safer: bool,
    #[serde(default)]
    pub description: String,
    /// Chance this exception is live for a given ride, rolled once per
    /// passenger by `roll_exception_liveness`. Unauthored means always.
    #[serde(default = "default_probability")]
    pub probability: f32,
    #[serde(rename = "requiredStage")]
    pub required_stage: Option<String>,
}

/// A guideline with detailed exception handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guideline {
    pub id: u32,
    pub title: String,
    pub description: String,
    /// How hard this read is, shown on the decision screen.
    #[serde(default)]
    pub difficulty: Difficulty,
    // `visible`, `type`, `isGuideline` and `defaultSafety` are authored on
    // guidelines and read by nothing. `visible` has no surface to act on —
    // there is no guideline list to hide an entry from; guidelines reach the
    // player through tells and the decision screen. The rest are redundant:
    // whether an exception is reachable is decided by which *rule* was drawn
    // for the shift, via `related_guideline_id`, so a guideline's own type
    // selects nothing. `isGuideline` is true on all eighteen and
    // `defaultSafety` is "safe" on all eighteen.
    #[serde(default)]
    pub exceptions: Vec<GuidelineException>,
    #[serde(rename = "followConsequences", default)]
    pub follow_consequences: Vec<Consequence>,
    #[serde(rename = "breakConsequences", default)]
    pub break_consequences: Vec<Consequence>,
    #[serde(rename = "exceptionRewards", default)]
    pub exception_rewards: Vec<Consequence>,
}
