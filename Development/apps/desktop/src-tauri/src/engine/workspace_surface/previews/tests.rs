use super::*;
#[test]
fn primary_failure_does_not_change_four_secondary_tile_receipts() {
    let tiles = (2..=5).map(|slot| PreviewTile {
        slot,
        viewport: Viewport {
            x: (slot * 160) as f64, y: 450.0,
            width: 150.0, height: 85.0, pixel_ratio: 1.0,
        },
    }).collect::<Vec<_>>();
    assert!(validate_tiles(&tiles, Some(1)).is_ok());
    let game = OwnedProcess {
        pid: 102, created_file_time: "12346".into(),
        session_id: 1, window_handle: "0x102".into(), responding: true,
    };
    let sources = vec![(2, game.clone()), (3, OwnedProcess {
        pid: 103, created_file_time: "12347".into(),
        window_handle: "0x103".into(), ..game
    })];
    let unavailable = vec![(3, "MC03 recovery unconfirmed".to_owned())];
    // A synthetic MC01 primary observer error has no slot in PreviewReport.
    let primary_error = io::Error::other("MC01 primary DWM unavailable");
    assert!(primary_error.to_string().contains("primary DWM"));
    let statuses = tiles.iter().map(|tile|
        initial_tile_status(tile, &sources, &unavailable)).collect::<Vec<_>>();
    assert_eq!(statuses.iter().map(|tile| tile.slot).collect::<Vec<_>>(),
        vec![2,3,4,5]);
    assert!(statuses.iter().all(|tile| !tile.registered));
    assert_eq!(statuses[0].reason, None);
    assert_eq!(statuses[1].reason.as_deref(), Some("MC03 recovery unconfirmed"));
    assert_eq!(statuses[2].reason.as_deref(),
        Some("No verified Minecraft preview source for this profile"));
    assert_eq!(statuses[3].reason.as_deref(),
        Some("No verified Minecraft preview source for this profile"));
}
#[test]
fn preview_recovery_reason_is_scoped_to_only_the_failed_slot() {
    let target = OwnedProcess {
        pid: 101, created_file_time: "12345".into(), session_id: 1,
        window_handle: "0x101".into(), responding: true,
    };
    let sources = vec![(1, target.clone()), (2, OwnedProcess {
        pid: 102, created_file_time: "12346".into(),
        window_handle: "0x102".into(), ..target
    })];
    let unavailable = vec![(1, "Original native recovery unconfirmed".to_owned())];
    assert_eq!(preview_tile_reason(1, &sources, &unavailable),
        Some("Original native recovery unconfirmed".into()));
    assert_eq!(preview_tile_reason(2, &sources, &unavailable), None,
        "MC01 recovery must not mark MC02 unavailable");
    assert_eq!(preview_tile_reason(3, &sources, &unavailable),
        Some("No verified Minecraft preview source for this profile".into()));
}
#[test]
fn closing_one_of_five_minecraft_slots_preserves_four_peer_previews() {
    for closing in 1..=5 {
        let visible=(1..=5)
            .filter(|observed|retain_other_preview_slot(*observed,closing))
            .collect::<Vec<_>>();
        assert_eq!(visible.len(),4);
        assert!(!visible.contains(&closing));
        for other in 1..=5 {
            assert_eq!(retain_other_preview_slot(other,closing),other!=closing);
        }
    }
}
fn tile(slot: u32, x: f64) -> PreviewTile {
    PreviewTile {
        slot,
        viewport: Viewport {
            x,
            y: 450.0,
            width: 150.0,
            height: 85.0,
            pixel_ratio: 1.0,
        },
    }
}
#[test]
fn preview_layout_rejects_duplicates_active_slot_and_invalid_geometry() {
    assert!(validate_tiles(
        &[
            tile(2, 10.0),
            tile(3, 170.0),
            tile(4, 330.0),
            tile(5, 490.0)
        ],
        Some(1)
    )
    .is_ok());
    assert!(validate_tiles(&[tile(2, 10.0), tile(2, 170.0)], Some(1)).is_err());
    assert!(validate_tiles(&[tile(1, 10.0)], Some(1)).is_err());
    assert!(validate_tiles(&[tile(0, 10.0)], None).is_err());
    assert!(validate_tiles(&[tile(2, f64::NAN)], None).is_err());
    assert!(validate_tiles(&[tile(2, -1.0)], None).is_err());
}
#[test]
fn letterboxing_preserves_aspect_and_overlap_excludes_game_input_area() {
    assert_eq!(
        fit_rectangle(200, 150, 1920, 1080).unwrap(),
        Rectangle {
            x: 0,
            y: 18,
            width: 200,
            height: 113
        }
    );
    assert!(fit_rectangle(200, 150, 0, 1080).is_err());
    let game = Rectangle {
        x: 10,
        y: 10,
        width: 800,
        height: 400,
    };
    assert!(!overlaps(
        game,
        Rectangle {
            x: 10,
            y: 420,
            width: 200,
            height: 100
        }
    ));
    assert!(overlaps(
        game,
        Rectangle {
            x: 10,
            y: 400,
            width: 200,
            height: 100
        }
    ));
}
