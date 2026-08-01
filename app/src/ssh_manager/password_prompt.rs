use lazy_static::lazy_static;
use regex::bytes::Regex;

const PASSWORD_PROMPT_PATTERN: &str = r"(?im)(password|passphrase)[^\n]*:\s*$";

/// OneKey 自动发送滑动窗口的预过滤正则:除密码类提示外,还放行用户名类提示
/// (`login` / `username` / `user` / `name` / `email` / `account`),使
/// `classify_prompt` 的 UsernameThenPassword 分类在生产路径可达。预过滤只做
/// 宽泛放行,精确分类交给 `classify_prompt` 按用户配置的关键词完成。
const ONEKEY_PROMPT_PATTERN: &str =
    r"(?im)(password|passphrase|login|username|user|name|email|account)[^\n]*:\s*$";

lazy_static! {
    static ref PASSWORD_PROMPT_REGEX: Regex =
        Regex::new(PASSWORD_PROMPT_PATTERN).expect("password prompt regex must compile");
    static ref ONEKEY_PROMPT_REGEX: Regex =
        Regex::new(ONEKEY_PROMPT_PATTERN).expect("onekey prompt regex must compile");
}

pub fn bytes_look_like_password_prompt(bytes: &[u8]) -> bool {
    PASSWORD_PROMPT_REGEX.is_match(bytes)
}

/// 判断 PTY 输出是否像"凭据类"提示行(密码或用户名)。
///
/// 与 `bytes_look_like_password_prompt` 区分:后者保持严格的 password /
/// passphrase 语义,供 SSH 密码注入(`secret_injector`)与 su 检测使用,避免
/// 把 SSH 密码错注入到 `login:` 用户名提示;本函数仅用于 OneKey 自动发送的
/// 滑动窗口预过滤。
pub fn bytes_look_like_onekey_prompt(bytes: &[u8]) -> bool {
    ONEKEY_PROMPT_REGEX.is_match(bytes)
}

#[cfg(test)]
#[path = "password_prompt_tests.rs"]
mod tests;
