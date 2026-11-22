use crate::i18n::{detect_system_lang, get_strings};
use crate::steam::{format_playtime, format_size, SteamGame};
use crate::widget_bindings::WidgetSize;
use serde_json::{json, Value};

/// Builds the Adaptive Card JSON template and data payload for the Steam Library widget.
pub fn build_adaptive_card(
    games: &[SteamGame],
    page: usize,
    size: WidgetSize,
    _status_msg: Option<&str>,
) -> (String, String) {
    let lang = detect_system_lang();
    let strings = get_strings(lang);

    let page_size = match size {
        WidgetSize::Small => 5,
        WidgetSize::Medium => 3,
        WidgetSize::Large => 6,
    };

    let total_games = games.len();
    let total_pages = if total_games == 0 {
        1
    } else {
        (total_games + page_size - 1) / page_size
    };
    let current_page = page.min(total_pages.saturating_sub(1));

    let start_idx = current_page * page_size;
    let end_idx = (start_idx + page_size).min(total_games);
    let visible_games = if start_idx < total_games {
        &games[start_idx..end_idx]
    } else {
        &[]
    };

    let mut body: Vec<Value> = Vec::new();

    // Header bar: Pagination controls (if needed) on the left + "Open Steam" and "Refresh" buttons side by side on the right
    let mut left_header_items: Vec<Value> = Vec::new();
    if total_pages > 1 {
        let mut page_actions: Vec<Value> = Vec::new();
        if current_page > 0 {
            page_actions.push(json!({
                "type": "Action.Execute",
                "title": strings.prev_page,
                "verb": "prev_page"
            }));
        }
        if current_page + 1 < total_pages {
            page_actions.push(json!({
                "type": "Action.Execute",
                "title": strings.next_page,
                "verb": "next_page"
            }));
        }
        left_header_items.push(json!({
            "type": "ActionSet",
            "spacing": "None",
            "actions": page_actions
        }));
    }

    body.push(json!({
        "type": "ColumnSet",
        "spacing": "None",
        "columns": [
            {
                "type": "Column",
                "width": "stretch",
                "verticalContentAlignment": "Center",
                "items": left_header_items
            },
            {
                "type": "Column",
                "width": "auto",
                "verticalContentAlignment": "Center",
                "items": [
                    {
                        "type": "ActionSet",
                        "spacing": "None",
                        "actions": [
                            {
                                "type": "Action.Execute",
                                "title": strings.open_steam,
                                "verb": "open_steam",
                                "tooltip": strings.open_steam
                            },
                            {
                                "type": "Action.Execute",
                                "title": "↻",
                                "verb": "refresh",
                                "tooltip": strings.refresh_tooltip
                            }
                        ]
                    }
                ]
            }
        ]
    }));

    if visible_games.is_empty() {
        body.push(json!({
            "type": "Container",
            "spacing": "Medium",
            "items": [
                {
                    "type": "TextBlock",
                    "text": strings.no_games,
                    "wrap": true,
                    "isSubtle": true,
                    "horizontalAlignment": "Center"
                }
            ]
        }));
    } else if size == WidgetSize::Small {
        // Small widget size: display several games in a single horizontal row with only their icons/logos, clickable to launch
        let mut icon_columns: Vec<Value> = Vec::new();
        for game in visible_games {
            let play_tip = format!("{} {}", strings.play_tooltip_prefix, game.name);
            icon_columns.push(json!({
                "type": "Column",
                "width": "auto",
                "spacing": "Medium",
                "verticalContentAlignment": "Center",
                "items": [
                    {
                        "type": "Container",
                        "style": "emphasis",
                        "selectAction": {
                            "type": "Action.Execute",
                            "verb": "launch",
                            "data": {
                                "appid": game.appid,
                                "name": game.name
                            },
                            "tooltip": play_tip
                        },
                        "items": [
                            {
                                "type": "Image",
                                "url": game.icon_data_uri,
                                "width": "48px",
                                "height": "48px",
                                "style": "RoundedCorners",
                                "altText": game.name,
                                "selectAction": {
                                    "type": "Action.Execute",
                                    "verb": "launch",
                                    "data": {
                                        "appid": game.appid,
                                        "name": game.name
                                    },
                                    "tooltip": format!("{} {}", strings.play_tooltip_prefix, game.name)
                                }
                            }
                        ]
                    }
                ]
            }));
        }

        body.push(json!({
            "type": "ColumnSet",
            "spacing": "Medium",
            "horizontalAlignment": "Center",
            "columns": icon_columns
        }));
    } else {
        // Medium and Large widget sizes: spacious list with icon, full name + "314,1 h | 321 MB", and compact blue "▶" play button
        for game in visible_games {
            let playtime_label = format_playtime(game.playtime_minutes, &strings);
            let size_label = format_size(game.size_on_disk, &strings);
            let meta_line = format!("{} | {}", playtime_label, size_label);
            let play_tip = format!("{} {}", strings.play_tooltip_prefix, game.name);

            body.push(json!({
                "type": "Container",
                "style": "emphasis",
                "spacing": "Medium",
                "items": [
                    {
                        "type": "ColumnSet",
                        "columns": [
                            {
                                "type": "Column",
                                "width": "auto",
                                "verticalContentAlignment": "Center",
                                "items": [
                                    {
                                        "type": "Image",
                                        "url": game.icon_data_uri,
                                        "width": "40px",
                                        "height": "40px",
                                        "style": "RoundedCorners",
                                        "altText": game.name
                                    }
                                ]
                            },
                            {
                                "type": "Column",
                                "width": "stretch",
                                "verticalContentAlignment": "Center",
                                "spacing": "Medium",
                                "items": [
                                    {
                                        "type": "TextBlock",
                                        "text": game.name,
                                        "weight": "Bolder",
                                        "wrap": false,
                                        "spacing": "None"
                                    },
                                    {
                                        "type": "TextBlock",
                                        "text": meta_line,
                                        "size": "Small",
                                        "isSubtle": true,
                                        "spacing": "Small",
                                        "wrap": false
                                    }
                                ]
                            },
                            {
                                "type": "Column",
                                "width": "auto",
                                "verticalContentAlignment": "Center",
                                "items": [
                                    {
                                        "type": "ActionSet",
                                        "spacing": "None",
                                        "actions": [
                                            {
                                                "type": "Action.Execute",
                                                "title": "▶",
                                                "verb": "launch",
                                                "data": {
                                                    "appid": game.appid,
                                                    "name": game.name
                                                },
                                                "style": "positive",
                                                "tooltip": play_tip
                                            }
                                        ]
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }));
        }
    }

    if total_pages > 1 {
        let page_indicator = format!("{} {}/{}", strings.page_label, current_page + 1, total_pages);
        body.push(json!({
            "type": "TextBlock",
            "text": page_indicator,
            "size": "Small",
            "isSubtle": true,
            "horizontalAlignment": "Right",
            "spacing": "Small"
        }));
    }

    let template = json!({
        "type": "AdaptiveCard",
        "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
        "version": "1.5",
        "body": body
    });

    (template.to_string(), "{}".to_string())
}
