use crate::youtube::YoutubeError;

pub fn parse(from: &str) -> Result<u64, YoutubeError> {
    let suffix = from.chars().last().unwrap_or(' ');
    let multiplier = match suffix {
        'K' | 'k' => 1_000,
        'M' => 1_000_000,
        'B' => 1_000_000_000,
        _ => 1
    } as f32;
    
    let replaced = from.replace(",", "");
    let replaced = if multiplier > 1.0 {
        replaced.replace(suffix, "")
    }
    else {
        replaced
    };

    match replaced.trim().parse::<f32>() {
        Ok(n) => Ok((n * multiplier) as u64),
        Err(err) => Err(YoutubeError::ParseError(err.to_string()))
    }
}

pub fn format(n: u64) -> String{
    let as_float = n as f32;
    match n {
        1_000_000_000..=u64::MAX => format!("{:.2} B", as_float / 1e9),
        1_000_000..=999_999_999 => format!("{:.2} M", as_float / 1e6),
        1_000..=999_999 => format!("{:.2} K", as_float / 1e3),
        _ => n.to_string()
    }
}