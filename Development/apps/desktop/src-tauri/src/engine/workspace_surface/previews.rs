//! DWM renders monitoring tiles and the unfocused main game into owned windows.
//! No screenshot loop, encoder, game input forwarding, or new lifecycle authority.
use super::{controller, native, OwnedProcess, Rectangle, Viewport};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreviewTile {
    pub slot: u32,
    pub viewport: Viewport,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileStatus {
    pub slot: u32,
    pub registered: bool,
    pub reason: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewReport {
    pub schema: u32,
    pub tiles: Vec<TileStatus>,
}

// DWM registration is observation, not permission to play or launch.
fn preview_tile_reason(
    slot: u32,
    profiles: &[(u32, OwnedProcess)],
    unavailable: &[(u32, String)],
) -> Option<String> {
    unavailable.iter().find(|(item, _)| *item == slot)
        .map(|(_, reason)| reason.clone())
        .or_else(|| (!profiles.iter().any(|(item, _)| *item == slot))
            .then(|| "No verified Minecraft preview source for this profile".into()))
}
// The primary DWM observer is separate from the requested secondary tiles.
// Share initial status evidence between the live and no-worker paths.
fn initial_tile_status(tile: &PreviewTile, profiles: &[(u32, OwnedProcess)],
    unavailable: &[(u32, String)]) -> TileStatus {
    TileStatus {
        slot: tile.slot,
        registered: false,
        reason: preview_tile_reason(tile.slot, profiles, unavailable),
    }
}
// A per-launcher close must never discard the remaining Minecraft tiles.
fn retain_other_preview_slot(observed:u32,closing:u32)->bool {
    observed!=closing
}

fn overlaps(a: Rectangle, b: Rectangle) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}
fn validate_tiles(tiles: &[PreviewTile], active: Option<u32>) -> io::Result<()> {
    let mut seen = std::collections::HashSet::new();
    if tiles.len() > 4 {
        return Err(io::Error::other("At most four preview tiles are supported"));
    }
    for tile in tiles {
        tile.viewport.validate_preview()?;
        if !(1..=5).contains(&tile.slot) || active == Some(tile.slot) || !seen.insert(tile.slot) {
            return Err(io::Error::other("Invalid or duplicate preview profile"));
        }
    }
    Ok(())
}

pub(super) fn configure(
    window: tauri::Window,
    viewport: Viewport,
    tiles: Vec<PreviewTile>,
    profiles: Vec<(u32, OwnedProcess)>,
) -> io::Result<PreviewReport> {
    #[cfg(windows)]
    {
        // Snapshot BEFORE potentially slow native preview adoption. A
        // CloseOne arriving during it invalidates this old Layout request.
        let issued_epochs = windows::snapshot_epochs();
        let manager = window.hwnd().map_err(io::Error::other)?.0 as usize as u64;
        let active = controller::active_preview_slot(manager)?;
        validate_tiles(&tiles, active)?;
        let main = native::rectangle(manager, viewport.validate()?)?;
        let rectangles = tiles
            .iter()
            .map(|tile| native::rectangle(manager, tile.viewport))
            .collect::<io::Result<Vec<_>>>()?;
        for (index, rect) in rectangles.iter().enumerate() {
            if overlaps(main, *rect)
                || rectangles[..index]
                    .iter()
                    .any(|other| overlaps(*other, *rect))
            {
                return Err(io::Error::other(
                    "Preview images must not overlap the game area or each other",
                ));
            }
        }
        // Full Clear or CloseOne may cancel this observation while we
        // validate manager geometry. Do not allocate a source roster or
        // admit native Park/Place work for a request already invalidated.
        windows::require_current_admission(&issued_epochs)?;
        let monitored=profiles.iter().filter(|(slot,_)|tiles.iter().any(|tile|tile.slot==*slot)).cloned().collect::<Vec<_>>();
        let unavailable = controller::adopt_preview_sources(manager, viewport, &monitored)?;
        // Adoption is owned by the existing native recovery controller and
        // may take time. Its returned report must not enqueue stale DWM
        // work if a CloseOne/Clear canceled this snapshot in the meantime.
        windows::require_current_admission(&issued_epochs)?;
        let primary = active.map(|slot| PreviewTile { slot, viewport });
        windows::apply(window, tiles, primary, profiles, unavailable, issued_epochs)
    }
    #[cfg(not(windows))]
    {
        let _ = (window, viewport, tiles, profiles);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Windows previews unavailable",
        ))
    }
}

#[cfg(windows)]
mod windows;

fn fit_rectangle(
    width: i32,
    height: i32,
    source_width: i32,
    source_height: i32,
) -> io::Result<Rectangle> {
    if width <= 0 || height <= 0 || source_width <= 0 || source_height <= 0 {
        return Err(io::Error::other("Preview source size unavailable"));
    }
    let scale = (width as f64 / source_width as f64).min(height as f64 / source_height as f64);
    let fitted_width = (source_width as f64 * scale).round().max(1.0) as i32;
    let fitted_height = (source_height as f64 * scale).round().max(1.0) as i32;
    Ok(Rectangle {
        x: (width - fitted_width) / 2,
        y: (height - fitted_height) / 2,
        width: fitted_width,
        height: fitted_height,
    })
}
pub(super) fn clear() -> io::Result<()> {
    #[cfg(windows)]
    {
        windows::clear()
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}
pub(super) fn clear_slot(slot:u32)->io::Result<()> {
    if !(1..=5).contains(&slot) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft preview slot"));
    }
    #[cfg(windows)]
    { windows::clear_slot(slot) }
    #[cfg(not(windows))]
    { Ok(()) }
}
pub(super) fn clear_primary() -> io::Result<()> {
    #[cfg(windows)]
    { windows::clear_primary() }
    #[cfg(not(windows))]
    { Ok(()) }
}
pub(super) fn refresh() {
    #[cfg(windows)]
    windows::refresh();
}

#[cfg(test)]
mod tests;
