pub(crate) fn split_exchange_symbol(symbol: &str) -> (Option<String>, String) {
    let symbol = symbol.trim();
    match symbol.split_once(':') {
        Some((exchange, name)) if !exchange.trim().is_empty() && !name.trim().is_empty() => (
            Some(exchange.trim().to_ascii_uppercase()),
            name.trim().to_ascii_uppercase(),
        ),
        _ => (None, symbol.to_ascii_uppercase()),
    }
}

pub(crate) fn bare_symbol(symbol: &str) -> String {
    symbol
        .split(':')
        .next_back()
        .unwrap_or(symbol)
        .to_ascii_uppercase()
}

pub(crate) fn strip_em(value: &str) -> String {
    value.replace("<em>", "").replace("</em>", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_symbol_compares_exchange_prefixed_inputs() {
        assert_eq!(bare_symbol("NASDAQ:AAPL"), bare_symbol("AAPL"));
        assert_eq!(bare_symbol("nyse:brk.b"), "BRK.B");
    }

    #[test]
    fn split_exchange_symbol_normalizes_optional_exchange() {
        assert_eq!(
            split_exchange_symbol(" nasdaq:aapl "),
            (Some("NASDAQ".to_string()), "AAPL".to_string())
        );
        assert_eq!(split_exchange_symbol("AAPL"), (None, "AAPL".to_string()));
    }
}
