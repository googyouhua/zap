use warp_quick_credential::{PromptTriggerRule, SendMode};

pub fn classify_prompt(text: &str, rules: &[PromptTriggerRule]) -> Option<SendMode> {
    let lower = text.to_lowercase();
    for rule in rules {
        if lower.contains(&rule.keyword.to_lowercase()) {
            return Some(rule.send_mode);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(keyword: &str, send_mode: SendMode) -> PromptTriggerRule {
        PromptTriggerRule {
            id: keyword.to_string(),
            keyword: keyword.to_string(),
            send_mode,
        }
    }

    #[test]
    fn test_classify_prompt_matches_keyword() {
        let rules = vec![
            make_rule("password", SendMode::PasswordOnly),
            make_rule("username", SendMode::UsernameThenPassword),
        ];
        assert_eq!(
            classify_prompt("Password:", &rules),
            Some(SendMode::PasswordOnly)
        );
        assert_eq!(
            classify_prompt("Enter Username:", &rules),
            Some(SendMode::UsernameThenPassword)
        );
    }

    #[test]
    fn test_classify_prompt_case_insensitive() {
        let rules = vec![make_rule("PASSWORD", SendMode::PasswordOnly)];
        assert_eq!(
            classify_prompt("password:", &rules),
            Some(SendMode::PasswordOnly)
        );
        assert_eq!(
            classify_prompt("Password:", &rules),
            Some(SendMode::PasswordOnly)
        );
        assert_eq!(
            classify_prompt("PASSWORD:", &rules),
            Some(SendMode::PasswordOnly)
        );
    }

    #[test]
    fn test_classify_prompt_no_match() {
        let rules = vec![
            make_rule("password", SendMode::PasswordOnly),
            make_rule("username", SendMode::UsernameThenPassword),
        ];
        assert_eq!(classify_prompt("Hello world", &rules), None);
        assert_eq!(classify_prompt("", &rules), None);
    }

    #[test]
    fn test_classify_prompt_first_match_wins() {
        let rules = vec![
            make_rule("password", SendMode::PasswordOnly),
            make_rule("passphrase", SendMode::PasswordOnly),
        ];
        assert_eq!(
            classify_prompt("Enter your passphrase:", &rules),
            Some(SendMode::PasswordOnly)
        );
    }

    #[test]
    fn test_classify_prompt_contained_in_longer_string() {
        let rules = vec![make_rule("email", SendMode::UsernameThenPassword)];
        assert_eq!(
            classify_prompt("Email address:", &rules),
            Some(SendMode::UsernameThenPassword)
        );
        assert_eq!(
            classify_prompt("enter email address:", &rules),
            Some(SendMode::UsernameThenPassword)
        );
    }
}
