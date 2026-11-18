//! Tolerant parsing of model output.
//!
//! Even with Ollama structured outputs enabled, small models occasionally wrap
//! JSON in markdown fences, add a sentence before it, or leave a trailing
//! comma. None of those should force an unnecessary hold, so parsing is
//! deliberately permissive while still rejecting non-JSON content.

use crate::error::Error;
use crate::types::TradeDecision;

pub fn parse_decision(content: &str) -> Result<TradeDecision, Error> {
    let trimmed = content.trim().trim_start_matches('\u{feff}').trim();
    if trimmed.is_empty() {
        return Err(Error::EmptyResponse);
    }

    let mut candidates: Vec<String> = vec![trimmed.to_string()];
    if let Some(fenced) = fenced_block(trimmed) {
        candidates.push(fenced.to_string());
    }
    if let Some(object) = extract_json_object(trimmed) {
        candidates.push(object.to_string());
    }
    if let Some(fenced) = fenced_block(trimmed)
        && let Some(object) = extract_json_object(fenced)
    {
        candidates.push(object.to_string());
    }

    // A second pass removes trailing commas from otherwise valid JSON.
    let with_sanitized: Vec<String> = candidates
        .iter()
        .map(|candidate| strip_trailing_commas(candidate))
        .filter(|candidate| !candidates.iter().any(|existing| existing == candidate))
        .collect();
    candidates.extend(with_sanitized);

    let mut last_error = None;
    for candidate in &candidates {
        match serde_json::from_str::<TradeDecision>(candidate) {
            Ok(decision) => return Ok(decision),
            Err(error) => last_error = Some(error),
        }
    }

    let detail = last_error
        .map(|error| error.to_string())
        .unwrap_or_else(|| "no JSON object found".to_string());
    Err(Error::Parse(format!(
        "{detail}; response began with: {}",
        snippet(trimmed)
    )))
}

fn fenced_block(text: &str) -> Option<&str> {
    let start = text.find("```")?;
    let rest = &text[start + 3..];
    let rest = rest
        .strip_prefix("json")
        .or_else(|| rest.strip_prefix("JSON"))
        .or_else(|| rest.strip_prefix("Json"))
        .unwrap_or(rest);
    let rest = rest.trim_start_matches(['\r', '\n', ' ', '\t']);
    let end = rest.find("```")?;
    let block = rest[..end].trim();
    if block.is_empty() { None } else { Some(block) }
}

fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in text[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(&text[start..start + offset + 1]);
                }
            }
            _ => {}
        }
    }

    None
}

fn strip_trailing_commas(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escaped = false;

    for (index, &ch) in chars.iter().enumerate() {
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            continue;
        }

        if ch == ',' {
            let next = chars[index + 1..]
                .iter()
                .find(|candidate| !candidate.is_whitespace());
            if matches!(next.copied(), Some('}') | Some(']')) {
                // Drop the comma; keep the whitespace that follows.
                continue;
            }
        }

        out.push(ch);
    }

    out
}

fn snippet(text: &str) -> String {
    const LIMIT: usize = 180;
    let mut snippet: String = text.chars().take(LIMIT).collect();
    if text.chars().count() > LIMIT {
        snippet.push('…');
    }
    snippet.replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, OrderType};

    #[test]
    fn parses_plain_json() {
        let decision = parse_decision(
            r#"{"action":"buy","confidence":0.72,"quantity":1.5,"stop_loss":95.0,"rationale":"trend"}"#,
        )
        .unwrap();
        assert_eq!(decision.action, Action::Buy);
        assert_eq!(decision.confidence, 0.72);
        assert_eq!(decision.quantity, Some(1.5));
    }

    #[test]
    fn parses_markdown_fence_with_prose() {
        let decision = parse_decision(
            "Here is my answer:\n```json\n{\"action\":\"sell\",\"confidence\":0.7,\"rationale\":\"weak\"}\n```\n",
        )
        .unwrap();
        assert_eq!(decision.action, Action::Sell);
    }

    #[test]
    fn parses_trailing_commas() {
        let decision =
            parse_decision(r#"{"action":"hold","confidence":0.4,"rationale":"mixed",}"#).unwrap();
        assert_eq!(decision.action, Action::Hold);
    }

    #[test]
    fn accepts_string_numbers_and_aliases() {
        let decision = parse_decision(
            r#"{"action":"LONG","confidence":"65%","quantity":"2","order_type":"limit","limit_price":"101.5","rationale":"breakout"}"#,
        )
        .unwrap();
        assert_eq!(decision.action, Action::Buy);
        assert!((decision.confidence - 0.65).abs() < 1e-9);
        assert_eq!(decision.quantity, Some(2.0));
        assert_eq!(decision.order_type, Some(OrderType::Limit));
        assert_eq!(decision.limit_price, Some(101.5));
    }

    #[test]
    fn parses_object_embedded_in_prose() {
        let decision = parse_decision(
            "I would hold here. {\"action\":\"hold\",\"confidence\":0.55,\"rationale\":\"spread\"} done.",
        )
        .unwrap();
        assert_eq!(decision.action, Action::Hold);
    }

    #[test]
    fn rejects_non_json() {
        let error = parse_decision("Buy BTC now because it is going up").unwrap_err();
        assert!(matches!(error, Error::Parse(_)));
    }

    #[test]
    fn rejects_empty() {
        assert!(matches!(parse_decision("   "), Err(Error::EmptyResponse)));
    }

    #[test]
    fn null_optionals_are_none() {
        let decision = parse_decision(
            r#"{"action":"hold","confidence":0.5,"quantity":null,"stop_loss":null,"rationale":"wait"}"#,
        )
        .unwrap();
        assert_eq!(decision.quantity, None);
        assert_eq!(decision.stop_loss, None);
    }
}
