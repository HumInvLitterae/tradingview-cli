use serde_json::{Value, json};

use tradingview_cdp::{KeyEvent, KeyEventType, MouseEvent, MouseEventType, RuntimeEvaluator};
use tradingview_core::{AppError, ErrorKind};

use super::super::common::require_finite;
use super::selectors::{number_field, ui_element_coordinates};

pub async fn ui_keyboard(
    runtime: &mut impl RuntimeEvaluator,
    key: &str,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
) -> Result<Value, AppError> {
    let mapping = key_mapping(key)?;
    let modifiers = modifier_mask(ctrl, shift, alt, meta);
    runtime
        .dispatch_key_event(KeyEvent {
            event_type: KeyEventType::KeyDown,
            key: mapping.key,
            code: mapping.code,
            windows_virtual_key_code: mapping.windows_virtual_key_code,
            modifiers,
        })
        .await?;
    runtime
        .dispatch_key_event(KeyEvent {
            event_type: KeyEventType::KeyUp,
            key: mapping.key,
            code: mapping.code,
            windows_virtual_key_code: mapping.windows_virtual_key_code,
            modifiers: 0,
        })
        .await?;
    Ok(json!({
        "key": mapping.key,
        "modifiers": modifier_names(ctrl, shift, alt, meta),
    }))
}

pub async fn ui_type(runtime: &mut impl RuntimeEvaluator, text: &str) -> Result<Value, AppError> {
    runtime.insert_text(text).await?;
    Ok(json!({
        "typed": text.chars().take(100).collect::<String>(),
        "length": text.chars().count(),
    }))
}

pub async fn ui_hover(
    runtime: &mut impl RuntimeEvaluator,
    by: &str,
    value: &str,
) -> Result<Value, AppError> {
    let coords = ui_element_coordinates(runtime, by, value).await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Moved,
            x: coords.x,
            y: coords.y,
            button: None,
            buttons: None,
            click_count: None,
            delta_x: None,
            delta_y: None,
        })
        .await?;
    Ok(json!({
        "hovered": {
            "by": by,
            "value": value,
            "tag": coords.tag,
            "x": coords.x,
            "y": coords.y
        }
    }))
}

pub async fn ui_scroll(
    runtime: &mut impl RuntimeEvaluator,
    direction: &str,
    amount: Option<f64>,
) -> Result<Value, AppError> {
    let direction = direction.trim().to_ascii_lowercase();
    if !matches!(direction.as_str(), "up" | "down" | "left" | "right") {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Scroll direction must be one of: up, down, left, right",
        ));
    }
    let amount = amount.unwrap_or(300.0);
    require_finite(amount, "amount")?;
    let center = runtime
        .evaluate(
            r#"
            (function() {
                var element = document.querySelector('[data-name="pane-canvas"]')
                    || document.querySelector('[class*="chart-container"]')
                    || document.querySelector('canvas');
                if (!element) return { x: window.innerWidth / 2, y: window.innerHeight / 2 };
                var rect = element.getBoundingClientRect();
                return { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
            })()
            "#,
            false,
        )
        .await?;
    let x = number_field(&center, "x")?;
    let y = number_field(&center, "y")?;
    let (delta_x, delta_y) = match direction.as_str() {
        "up" => (0.0, -amount),
        "down" => (0.0, amount),
        "left" => (-amount, 0.0),
        "right" => (amount, 0.0),
        _ => unreachable!("direction validated"),
    };
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Wheel,
            x,
            y,
            button: None,
            buttons: None,
            click_count: None,
            delta_x: Some(delta_x),
            delta_y: Some(delta_y),
        })
        .await?;
    Ok(json!({
        "direction": direction,
        "amount": amount,
        "x": x,
        "y": y,
    }))
}

pub async fn ui_mouse(
    runtime: &mut impl RuntimeEvaluator,
    x: f64,
    y: f64,
    right: bool,
    double: bool,
) -> Result<Value, AppError> {
    require_finite(x, "x")?;
    require_finite(y, "y")?;
    let button = if right { "right" } else { "left" };
    let buttons = if right { 2 } else { 1 };
    dispatch_mouse_click(runtime, x, y, button, buttons, 1).await?;
    if double {
        dispatch_mouse_click(runtime, x, y, button, buttons, 2).await?;
    }
    Ok(json!({
        "x": x,
        "y": y,
        "button": button,
        "double_click": double,
    }))
}

async fn dispatch_mouse_click(
    runtime: &mut impl RuntimeEvaluator,
    x: f64,
    y: f64,
    button: &'static str,
    buttons: i64,
    click_count: i64,
) -> Result<(), AppError> {
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Moved,
            x,
            y,
            button: None,
            buttons: None,
            click_count: None,
            delta_x: None,
            delta_y: None,
        })
        .await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Pressed,
            x,
            y,
            button: Some(button),
            buttons: Some(buttons),
            click_count: Some(click_count),
            delta_x: None,
            delta_y: None,
        })
        .await?;
    runtime
        .dispatch_mouse_event(MouseEvent {
            event_type: MouseEventType::Released,
            x,
            y,
            button: Some(button),
            buttons: Some(0),
            click_count: Some(click_count),
            delta_x: None,
            delta_y: None,
        })
        .await
}

fn modifier_mask(ctrl: bool, shift: bool, alt: bool, meta: bool) -> i64 {
    let mut mask = 0;
    if alt {
        mask |= 1;
    }
    if ctrl {
        mask |= 2;
    }
    if meta {
        mask |= 4;
    }
    if shift {
        mask |= 8;
    }
    mask
}

fn modifier_names(ctrl: bool, shift: bool, alt: bool, meta: bool) -> Vec<&'static str> {
    let mut names = Vec::new();
    if ctrl {
        names.push("ctrl");
    }
    if shift {
        names.push("shift");
    }
    if alt {
        names.push("alt");
    }
    if meta {
        names.push("meta");
    }
    names
}

struct KeyMapping {
    key: &'static str,
    code: &'static str,
    windows_virtual_key_code: i64,
}

fn key_mapping(key: &str) -> Result<KeyMapping, AppError> {
    let normalized = key.trim();
    if normalized.is_empty() {
        return Err(AppError::new(
            ErrorKind::Validation,
            "Key must not be empty",
        ));
    }
    let mapping = match normalized {
        "Enter" => ("Enter", "Enter", 13),
        "Escape" => ("Escape", "Escape", 27),
        "Tab" => ("Tab", "Tab", 9),
        "Backspace" => ("Backspace", "Backspace", 8),
        "Delete" => ("Delete", "Delete", 46),
        "ArrowUp" => ("ArrowUp", "ArrowUp", 38),
        "ArrowDown" => ("ArrowDown", "ArrowDown", 40),
        "ArrowLeft" => ("ArrowLeft", "ArrowLeft", 37),
        "ArrowRight" => ("ArrowRight", "ArrowRight", 39),
        "Space" => ("Space", "Space", 32),
        "Home" => ("Home", "Home", 36),
        "End" => ("End", "End", 35),
        "PageUp" => ("PageUp", "PageUp", 33),
        "PageDown" => ("PageDown", "PageDown", 34),
        "F1" => ("F1", "F1", 112),
        "F2" => ("F2", "F2", 113),
        "F5" => ("F5", "F5", 116),
        "a" | "A" => ("a", "KeyA", 65),
        "b" | "B" => ("b", "KeyB", 66),
        "c" | "C" => ("c", "KeyC", 67),
        "d" | "D" => ("d", "KeyD", 68),
        "e" | "E" => ("e", "KeyE", 69),
        "f" | "F" => ("f", "KeyF", 70),
        "g" | "G" => ("g", "KeyG", 71),
        "h" | "H" => ("h", "KeyH", 72),
        "i" | "I" => ("i", "KeyI", 73),
        "j" | "J" => ("j", "KeyJ", 74),
        "k" | "K" => ("k", "KeyK", 75),
        "l" | "L" => ("l", "KeyL", 76),
        "m" | "M" => ("m", "KeyM", 77),
        "n" | "N" => ("n", "KeyN", 78),
        "o" | "O" => ("o", "KeyO", 79),
        "p" | "P" => ("p", "KeyP", 80),
        "q" | "Q" => ("q", "KeyQ", 81),
        "r" | "R" => ("r", "KeyR", 82),
        "s" | "S" => ("s", "KeyS", 83),
        "t" | "T" => ("t", "KeyT", 84),
        "u" | "U" => ("u", "KeyU", 85),
        "v" | "V" => ("v", "KeyV", 86),
        "w" | "W" => ("w", "KeyW", 87),
        "x" | "X" => ("x", "KeyX", 88),
        "y" | "Y" => ("y", "KeyY", 89),
        "z" | "Z" => ("z", "KeyZ", 90),
        _ => {
            return Err(AppError::new(
                ErrorKind::Validation,
                format!("Unsupported key: {normalized}"),
            ));
        }
    };
    Ok(KeyMapping {
        key: mapping.0,
        code: mapping.1,
        windows_virtual_key_code: mapping.2,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::super::test_support::FakeRuntime;
    use super::*;

    #[tokio::test]
    async fn ui_keyboard_dispatches_key_events() {
        let mut runtime = FakeRuntime::new([]);

        let result = ui_keyboard(&mut runtime, "Escape", true, false, false, false)
            .await
            .unwrap();

        assert_eq!(result["key"], "Escape");
        assert_eq!(runtime.key_events.len(), 2);
        assert_eq!(runtime.key_events[0].event_type, KeyEventType::KeyDown);
        assert_eq!(runtime.key_events[0].modifiers, 2);
        assert_eq!(runtime.key_events[1].event_type, KeyEventType::KeyUp);
    }

    #[tokio::test]
    async fn ui_type_inserts_text() {
        let mut runtime = FakeRuntime::new([]);

        let result = ui_type(&mut runtime, "hello").await.unwrap();

        assert_eq!(result["length"], 5);
        assert_eq!(runtime.inserted_text, vec!["hello"]);
    }

    #[tokio::test]
    async fn ui_hover_moves_to_element_center() {
        let mut runtime = FakeRuntime::new([json!({"x": 10.0, "y": 20.0, "tag": "button"})]);

        let result = ui_hover(&mut runtime, "text", "Alerts").await.unwrap();

        assert_eq!(result["hovered"]["x"], 10.0);
        assert_eq!(runtime.mouse_events.len(), 1);
        assert_eq!(runtime.mouse_events[0].event_type, MouseEventType::Moved);
    }

    #[tokio::test]
    async fn ui_scroll_dispatches_wheel_event() {
        let mut runtime = FakeRuntime::new([json!({"x": 100.0, "y": 200.0})]);

        let result = ui_scroll(&mut runtime, "up", Some(150.0)).await.unwrap();

        assert_eq!(result["direction"], "up");
        assert_eq!(runtime.mouse_events.len(), 1);
        assert_eq!(runtime.mouse_events[0].event_type, MouseEventType::Wheel);
        assert_eq!(runtime.mouse_events[0].delta_y, Some(-150.0));
    }

    #[tokio::test]
    async fn ui_mouse_dispatches_click_events() {
        let mut runtime = FakeRuntime::new([]);

        let result = ui_mouse(&mut runtime, 1.0, 2.0, true, false).await.unwrap();

        assert_eq!(result["button"], "right");
        assert_eq!(runtime.mouse_events.len(), 3);
        assert_eq!(runtime.mouse_events[1].button, Some("right"));
    }
}
