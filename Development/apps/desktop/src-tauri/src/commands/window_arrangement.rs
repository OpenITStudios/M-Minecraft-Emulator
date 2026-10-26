use crate::engine::{
    screen_overlay::{self, OverlayPosition, ScreenOverlayItem},
    window_arrangement::{self, DisplayInfo, WindowArrangementResult, WindowLayoutRequest},
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayPreference {
    pub enabled: bool,
    pub show_screen_number: bool,
    pub show_label: bool,
    pub position: OverlayPosition,
    pub opacity: f32,
    pub labels: HashMap<String, String>,
}

fn validate_overlay_preference(preference: &OverlayPreference) -> Result<(), String> {
    if !preference.opacity.is_finite() || !(0.35..=1.0).contains(&preference.opacity) {
        return Err("Screen Overlay opacity must be between 0.35 and 1.0.".into());
    }
    for (id, label) in &preference.labels {
        if !matches!(id.as_str(), "Native" | "Virtual-01" | "Virtual-02" | "Virtual-03") {
            return Err(format!("Unknown Screen Overlay client label: {id}"));
        }
        if label.chars().count() > 32 {
            return Err(format!("Screen Overlay label for {id} exceeds 32 characters."));
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyWindowLayoutRequest {
    pub layout: WindowLayoutRequest,
    pub overlay: OverlayPreference,
    #[serde(default)]
    pub identify: bool,
}

#[tauri::command]
pub async fn window_displays() -> Result<Vec<DisplayInfo>, String> {
    tauri::async_runtime::spawn_blocking(window_arrangement::displays)
        .await
        .map_err(|error| format!("Display discovery task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn window_apply_layout(request: ApplyWindowLayoutRequest) -> Result<WindowArrangementResult, String> {
    validate_overlay_preference(&request.overlay)?;
    tauri::async_runtime::spawn_blocking(move || {
        let execution = window_arrangement::arrange_with_slots(request.layout)?;
        let items = execution.slots.iter().enumerate().map(|(index, slot)| ScreenOverlayItem {
            screen_number: index + 1,
            label: request.overlay.labels.get(&slot.id).cloned().unwrap_or_else(|| slot.id.clone()),
            x: slot.rect.x,
            y: slot.rect.y,
            width: slot.rect.width,
            height: slot.rect.height,
        }).collect();
        let mut result = execution.result;
        match screen_overlay::apply(screen_overlay::ScreenOverlayRequest {
            enabled: request.overlay.enabled,
            show_screen_number: request.overlay.show_screen_number,
            show_label: request.overlay.show_label,
            position: request.overlay.position,
            opacity: request.overlay.opacity,
            identify: request.identify,
            items,
        }) {
            Ok(()) => result.overlay_applied = Some(request.overlay.enabled),
            Err(error) => {
                result.overlay_applied = Some(false);
                result.overlay_warning = Some(error.to_string());
            }
        }
        Ok::<_, std::io::Error>(result)
    })
    .await
    .map_err(|error| format!("Window Layout task failed: {error}"))?
    .map_err(|error| error.to_string())
}


#[cfg(test)]
mod tests {
    use super::{validate_overlay_preference, OverlayPreference};
    use crate::engine::screen_overlay::OverlayPosition;
    use std::collections::HashMap;

    #[test]
    fn overlay_preference_rejects_unknown_clients_and_unbounded_labels() {
        let mut labels = HashMap::from([("Native".into(), "This PC".into())]);
        let valid = OverlayPreference {
            enabled: true,
            show_screen_number: true,
            show_label: true,
            position: OverlayPosition::TopLeft,
            opacity: 0.88,
            labels: labels.clone(),
        };
        assert!(validate_overlay_preference(&valid).is_ok());
        labels.insert("Unknown".into(), "Other".into());
        assert!(validate_overlay_preference(&OverlayPreference { labels, ..valid }).is_err());

        let long_label = OverlayPreference {
            enabled: true,
            show_screen_number: true,
            show_label: true,
            position: OverlayPosition::TopLeft,
            opacity: 0.88,
            labels: HashMap::from([("Native".into(), "x".repeat(33))]),
        };
        assert!(validate_overlay_preference(&long_label).is_err());
    }
}
