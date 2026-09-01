use agent_client_protocol as acp;
use serde::Serialize;
use xai_grok_sampling_types::{ReasoningEffort, ReasoningEffortOption};

use crate::session::unified_list::SessionKind;

pub(crate) const REASONING_EFFORT_CONFIG_ID: &str = "reasoning_effort";

pub(crate) const SELECTABLE_REASONING_EFFORTS: [ReasoningEffort; 5] = [
    ReasoningEffort::Minimal,
    ReasoningEffort::Low,
    ReasoningEffort::Medium,
    ReasoningEffort::High,
    ReasoningEffort::Xhigh,
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionConfigOption {
    pub id: String,
    pub category: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GrokSessionDetail {
    pub session_id: String,
    pub kind: String,
    pub cwd: String,
    pub current_model_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl GrokSessionDetail {
    pub(crate) fn build(
        session_id: String,
        cwd: String,
        current_model_id: String,
        title: Option<String>,
    ) -> Self {
        Self {
            session_id,
            kind: SessionKind::Build.as_str().to_string(),
            cwd,
            current_model_id,
            title,
        }
    }
}

fn effort_label(effort: ReasoningEffort) -> String {
    match effort {
        ReasoningEffort::None => "None",
        ReasoningEffort::Minimal => "Minimal",
        ReasoningEffort::Low => "Low",
        ReasoningEffort::Medium => "Medium",
        ReasoningEffort::High => "High",
        ReasoningEffort::Xhigh => "X-High",
        ReasoningEffort::Max => "Max",
    }
    .to_string()
}

/// The built-in session-picker modes used when the model has no server list.
/// Reproduces the historical five rows and their labels.
pub(crate) fn legacy_session_effort_options() -> Vec<ReasoningEffortOption> {
    SELECTABLE_REASONING_EFFORTS
        .iter()
        .map(|&effort| ReasoningEffortOption {
            id: effort.as_str().to_string(),
            value: effort,
            label: effort_label(effort),
            description: None,
            default: false,
        })
        .collect()
}

pub(crate) fn resolve_reasoning_effort_value(
    options: &[ReasoningEffortOption],
    value_id: &str,
) -> Option<ReasoningEffort> {
    options
        .iter()
        .find(|option| option.id == value_id)
        .map(|option| option.value)
}

pub(crate) fn build_acp_reasoning_effort_option(
    options: &[ReasoningEffortOption],
    current_effort: Option<ReasoningEffort>,
) -> Option<acp::SessionConfigOption> {
    let current = current_effort
        .and_then(|effort| options.iter().find(|option| option.value == effort))
        .or_else(|| options.iter().find(|option| option.default))?;
    let values = options
        .iter()
        .map(|option| {
            acp::SessionConfigSelectOption::new(option.id.clone(), option.label.clone())
                .description(option.description.clone())
        })
        .collect::<Vec<_>>();
    Some(
        acp::SessionConfigOption::select(
            REASONING_EFFORT_CONFIG_ID,
            "Reasoning Effort",
            current.id.clone(),
            values,
        )
        .category(acp::SessionConfigOptionCategory::ThoughtLevel),
    )
}

pub(crate) fn build_session_config_options(
    available_models: &[acp::ModelInfo],
    current_model_id: &acp::ModelId,
    effort_options: &[ReasoningEffortOption],
    current_effort: Option<ReasoningEffort>,
) -> Vec<SessionConfigOption> {
    let mut options = Vec::with_capacity(available_models.len() + effort_options.len());

    for model in available_models {
        let label = if model.name.is_empty() {
            model.model_id.0.to_string()
        } else {
            model.name.clone()
        };
        options.push(SessionConfigOption {
            id: model.model_id.0.to_string(),
            category: "model".to_string(),
            label,
            description: None,
            selected: model.model_id == *current_model_id,
        });
    }

    for effort in effort_options {
        options.push(SessionConfigOption {
            id: effort.id.clone(),
            category: "mode".to_string(),
            label: effort.label.clone(),
            description: effort.description.clone(),
            selected: Some(effort.value) == current_effort,
        });
    }

    options
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(id: &'static str, name: &str) -> acp::ModelInfo {
        acp::ModelInfo::new(acp::ModelId::new(id), name.to_string())
    }

    #[test]
    fn options_have_one_selected_model_and_a_mode_per_effort() {
        let models = [
            model("grok-build", "Grok Build"),
            model("grok-4.5", "Grok 4.5"),
        ];
        let current = acp::ModelId::from("grok-build");
        let opts = build_session_config_options(
            &models,
            &current,
            &legacy_session_effort_options(),
            Some(ReasoningEffort::High),
        );

        let model_opts: Vec<_> = opts.iter().filter(|o| o.category == "model").collect();
        assert_eq!(model_opts.len(), 2);
        let selected_models: Vec<_> = model_opts.iter().filter(|o| o.selected).collect();
        assert_eq!(selected_models.len(), 1);
        assert_eq!(selected_models[0].id, "grok-build");

        let mode_opts: Vec<_> = opts.iter().filter(|o| o.category == "mode").collect();
        assert_eq!(mode_opts.len(), SELECTABLE_REASONING_EFFORTS.len());
        let selected_modes: Vec<_> = mode_opts.iter().filter(|o| o.selected).collect();
        assert_eq!(selected_modes.len(), 1);
        assert_eq!(selected_modes[0].id, "high");
        assert_eq!(selected_modes[0].label, "High");
    }

    #[test]
    fn none_effort_is_not_a_user_selectable_mode() {
        assert!(!SELECTABLE_REASONING_EFFORTS.contains(&ReasoningEffort::None));
        let models = [model("grok-build", "Grok Build")];
        let current = acp::ModelId::from("grok-build");
        let opts = build_session_config_options(
            &models,
            &current,
            &legacy_session_effort_options(),
            Some(ReasoningEffort::None),
        );
        let modes: Vec<_> = opts.iter().filter(|o| o.category == "mode").collect();
        assert!(modes.iter().all(|o| o.id != "none"));
        assert!(modes.iter().all(|o| !o.selected));
    }

    #[test]
    fn no_mode_options_when_model_lacks_effort_support() {
        let models = [model("grok-build", "Grok Build")];
        let current = acp::ModelId::from("grok-build");
        let opts = build_session_config_options(&models, &current, &[], None);
        assert_eq!(opts.len(), 1);
        assert!(opts.iter().all(|o| o.category == "model"));
    }

    #[test]
    fn model_label_falls_back_to_id_when_name_empty() {
        let models = [model("grok-build", "")];
        let current = acp::ModelId::from("grok-build");
        let opts = build_session_config_options(&models, &current, &[], None);
        assert_eq!(opts[0].label, "grok-build");
    }

    #[test]
    fn session_config_option_serializes_camel_case() {
        let opt = SessionConfigOption {
            id: "grok-build".to_string(),
            category: "model".to_string(),
            label: "Grok Build".to_string(),
            description: None,
            selected: true,
        };
        let v = serde_json::to_value(&opt).expect("serialize");
        assert_eq!(v["id"], "grok-build");
        assert_eq!(v["category"], "model");
        assert_eq!(v["label"], "Grok Build");
        assert_eq!(v["selected"], true);
        assert!(v.get("description").is_none());
    }

    #[test]
    fn acp_reasoning_effort_option_uses_catalog_ids_and_thought_category() {
        let efforts = vec![
            ReasoningEffortOption {
                id: "balanced".to_string(),
                value: ReasoningEffort::Medium,
                label: "Balanced".to_string(),
                description: None,
                default: false,
            },
            ReasoningEffortOption {
                id: "deep".to_string(),
                value: ReasoningEffort::Xhigh,
                label: "Deep".to_string(),
                description: Some("Maximum depth".to_string()),
                default: true,
            },
        ];
        let option = build_acp_reasoning_effort_option(&efforts, Some(ReasoningEffort::Xhigh))
            .expect("supported effort selector");
        assert_eq!(option.id.0.as_ref(), REASONING_EFFORT_CONFIG_ID);
        assert_eq!(
            option.category,
            Some(acp::SessionConfigOptionCategory::ThoughtLevel)
        );
        let wire = serde_json::to_value(&option).unwrap();
        assert_eq!(wire["id"], REASONING_EFFORT_CONFIG_ID);
        assert_eq!(wire["category"], "thought_level");
        assert_eq!(wire["type"], "select");
        assert_eq!(wire["currentValue"], "deep");
        let acp::SessionConfigKind::Select(select) = option.kind else {
            panic!("reasoning effort must be a select option");
        };
        assert_eq!(select.current_value.0.as_ref(), "deep");
        let acp::SessionConfigSelectOptions::Ungrouped(values) = select.options else {
            panic!("reasoning effort values must be ungrouped");
        };
        assert_eq!(
            values
                .iter()
                .map(|value| value.value.0.as_ref())
                .collect::<Vec<_>>(),
            ["balanced", "deep"]
        );
        assert_eq!(values[1].description.as_deref(), Some("Maximum depth"));
    }

    #[test]
    fn acp_reasoning_effort_value_maps_catalog_id_to_canonical_effort() {
        let efforts = vec![ReasoningEffortOption {
            id: "deep".to_string(),
            value: ReasoningEffort::Xhigh,
            label: "Deep".to_string(),
            description: None,
            default: true,
        }];
        assert_eq!(
            resolve_reasoning_effort_value(&efforts, "deep"),
            Some(ReasoningEffort::Xhigh)
        );
        assert_eq!(resolve_reasoning_effort_value(&efforts, "xhigh"), None);
    }

    #[test]
    fn grok_session_detail_serializes_camel_case() {
        let detail = GrokSessionDetail::build(
            "sess-1".to_string(),
            "/Users/me/xai".to_string(),
            "grok-build".to_string(),
            None,
        );
        let v = serde_json::to_value(&detail).expect("serialize");
        assert_eq!(v["sessionId"], "sess-1");
        assert_eq!(v["kind"], "build");
        assert_eq!(v["cwd"], "/Users/me/xai");
        assert_eq!(v["currentModelId"], "grok-build");
        assert!(v.get("title").is_none());
    }
}
