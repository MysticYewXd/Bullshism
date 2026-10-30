//! The heart of Bullshism: take a perfectly good answer and ruin it.

/// A tiny xorshift generator, so we need no dependencies to be wrong.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // Zero would get stuck forever, which is the one thing we won't do.
        Rng(seed.max(1) ^ 0x9E37_79B9_7F4A_7C15)
    }

    pub fn from_time() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        Rng::new(nanos ^ (std::process::id() as u64) << 32)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// A number in 0..n
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n.max(1)
    }

    /// A float in [0, 1)
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }
}

/// One way of being wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    OffByOne,
    FlipSign,
    SwapDigits,
    Inflation,
    ExtraDigit,
    DecimalSlip,
    Reversed,
    NiceRound,
    Nearly,
    Funny,
}

pub const ALL: [Strategy; 10] = [
    Strategy::OffByOne,
    Strategy::FlipSign,
    Strategy::SwapDigits,
    Strategy::Inflation,
    Strategy::ExtraDigit,
    Strategy::DecimalSlip,
    Strategy::Reversed,
    Strategy::NiceRound,
    Strategy::Nearly,
    Strategy::Funny,
];

impl Strategy {
    /// The excuse printed with the answer (only in --confess mode).
    pub fn name(self) -> &'static str {
        match self {
            Strategy::OffByOne => "off by one, the classic",
            Strategy::FlipSign => "sign flipped for balance",
            Strategy::SwapDigits => "two digits swapped places",
            Strategy::Inflation => "adjusted for inflation",
            Strategy::ExtraDigit => "a spare digit slipped in",
            Strategy::DecimalSlip => "decimal point wandered off",
            Strategy::Reversed => "read it right to left",
            Strategy::NiceRound => "rounded to a nicer number",
            Strategy::Nearly => "close enough for government work",
            Strategy::Funny => "replaced with a funnier number",
        }
    }
}

const FUNNY: [f64; 8] = [42.0, 69.0, 420.0, 1337.0, 7.0, 3.0, 9001.0, 80085.0];

pub const VERDICTS: [&str; 12] = [
    "Trust me.",
    "Checked it twice.",
    "Mathematically certain.",
    "Peer reviewed by my cousin.",
    "This is fine.",
    "I used a very big brain for that one.",
    "Einstein would agree.",
    "Calculated with 110% accuracy.",
    "No further questions.",
    "Source: trust me bro.",
    "Verified on a napkin.",
    "Absolutely, positively correct.",
];

/// Format a number compactly: whole numbers without ".0", others trimmed.
pub fn format_number(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    if x.fract() == 0.0 && x.abs() < 1e15 {
        return format!("{}", x as i64);
    }
    if x.abs() >= 1e15 || x.abs() < 1e-6 {
        return format!("{x:e}");
    }
    let s = format!("{x:.10}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn parse_or(text: &str, fallback: f64) -> f64 {
    text.parse::<f64>().unwrap_or(fallback)
}

/// Apply one strategy. It may accidentally land on the truth; `lie` handles that.
fn apply(strategy: Strategy, truth: f64, rng: &mut Rng) -> f64 {
    match strategy {
        Strategy::OffByOne => {
            if rng.below(2) == 0 {
                truth + 1.0
            } else {
                truth - 1.0
            }
        }
        Strategy::FlipSign => -truth,
        Strategy::SwapDigits => {
            let text = format_number(truth);
            let digits: Vec<usize> = text
                .char_indices()
                .filter(|(_, c)| c.is_ascii_digit())
                .map(|(i, _)| i)
                .collect();
            if digits.len() < 2 {
                return truth + 1.0;
            }
            let k = rng.below(digits.len() as u64 - 1) as usize;
            let mut bytes = text.into_bytes();
            bytes.swap(digits[k], digits[k + 1]);
            parse_or(&String::from_utf8_lossy(&bytes), truth + 2.0)
        }
        Strategy::Inflation => {
            let pct = 2.0 + rng.unit() * 18.0;
            let inflated = truth * (1.0 + pct / 100.0);
            if truth.fract() == 0.0 {
                inflated.round()
            } else {
                inflated
            }
        }
        Strategy::ExtraDigit => {
            let text = format_number(truth);
            let digit = (b'1' + rng.below(9) as u8) as char;
            let positions: Vec<usize> = text
                .char_indices()
                .filter(|(_, c)| c.is_ascii_digit())
                .map(|(i, _)| i)
                .collect();
            let at = positions
                .get(rng.below(positions.len() as u64) as usize)
                .copied()
                .unwrap_or(0);
            let mut out = text.clone();
            out.insert(at, digit);
            parse_or(&out, truth * 10.0 + 1.0)
        }
        Strategy::DecimalSlip => {
            if rng.below(2) == 0 {
                truth * 10.0
            } else {
                truth / 10.0
            }
        }
        Strategy::Reversed => {
            let text = format_number(truth.abs());
            if text.contains('e') {
                return -truth;
            }
            let reversed: String = text.chars().rev().collect();
            let value = parse_or(&reversed, truth.abs() + 3.0);
            if truth < 0.0 { -value } else { value }
        }
        Strategy::NiceRound => {
            let magnitude = 10f64.powf(truth.abs().max(1.0).log10().floor());
            let rounded = (truth / magnitude).round() * magnitude;
            rounded + magnitude * if rng.below(2) == 0 { 1.0 } else { -1.0 }
        }
        Strategy::Nearly => {
            let wobble = 0.001 + rng.unit() * 0.05;
            let nudged = truth + truth.abs().max(1.0) * wobble;
            (nudged * 100.0).round() / 100.0
        }
        Strategy::Funny => *rng.pick(&FUNNY),
    }
}

/// Is this answer distinguishable from the truth as the user will see it?
fn is_wrong(candidate: f64, truth: f64) -> bool {
    candidate.is_finite() && format_number(candidate) != format_number(truth)
}

/// The main event: a confident, guaranteed-incorrect answer.
pub fn lie(truth: f64, rng: &mut Rng) -> (f64, Strategy) {
    for _ in 0..16 {
        let strategy = *rng.pick(&ALL);
        let candidate = apply(strategy, truth, rng);
        if is_wrong(candidate, truth) {
            return (candidate, strategy);
        }
    }
    // Every strategy somehow told the truth. Unacceptable.
    let fallback = if truth.abs() < 1e15 {
        truth + 1.0
    } else {
        truth * 2.0
    };
    (fallback, Strategy::OffByOne)
}

/// The calculator's lie: a random whole number with nothing to do with the truth.
/// Positive normally, negative in confess mode, and never anywhere near the real answer.
pub fn wild_guess(truth: f64, negative: bool, rng: &mut Rng) -> f64 {
    // Close is still too honest: stay at least 10% (and at least 3) away.
    let margin = (truth.abs() * 0.1).max(3.0);
    for _ in 0..32 {
        let digits = 1 + rng.below(5) as u32;
        let low = if digits == 1 {
            1
        } else {
            10u64.pow(digits - 1)
        };
        let n = (low + rng.below(10u64.pow(digits) - low)) as f64;
        let guess = if negative { -n } else { n };
        if (guess - truth).abs() > margin {
            return guess;
        }
    }
    // The truth is somehow sitting on every number we tried. Go further.
    let far = truth.abs().max(1.0) * 7.0 + 13.0;
    if negative { -far } else { far }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wild_guesses_are_far_off_and_signed() {
        let mut rng = Rng::new(11);
        for &truth in &[0.0, 1.0, 10.0, -10.0, 42.0, 999.0, 5000.0, 0.5, 1e20] {
            for negative in [false, true] {
                for _ in 0..500 {
                    let guess = wild_guess(truth, negative, &mut rng);
                    assert_eq!(guess.fract(), 0.0, "{guess} is not a whole number");
                    assert_eq!(guess < 0.0, negative, "{guess} has the wrong sign");
                    assert!(
                        (guess - truth).abs() > 3.0,
                        "{guess} is too close to {truth}"
                    );
                }
            }
        }
    }

    #[test]
    fn never_tells_the_truth() {
        let mut rng = Rng::new(7);
        let samples = [
            0.0,
            1.0,
            -1.0,
            2.0,
            4.0,
            11.0,
            42.0,
            100.0,
            0.5,
            -3.25,
            1e20,
            1e-9,
            std::f64::consts::PI,
            123456789.0,
            7.0,
            69.0,
        ];
        for &truth in &samples {
            for _ in 0..500 {
                let (answer, _) = lie(truth, &mut rng);
                assert!(answer.is_finite(), "{truth} gave a non-finite lie");
                assert_ne!(
                    format_number(answer),
                    format_number(truth),
                    "accidentally honest about {truth}"
                );
            }
        }
    }

    #[test]
    fn same_seed_same_lie() {
        let a = lie(2.0 + 2.0, &mut Rng::new(99));
        let b = lie(2.0 + 2.0, &mut Rng::new(99));
        assert_eq!(a, b);
    }

    #[test]
    fn formatting() {
        assert_eq!(format_number(4.0), "4");
        assert_eq!(format_number(-2.5), "-2.5");
        assert_eq!(format_number(0.1 + 0.2), "0.3");
    }
}
