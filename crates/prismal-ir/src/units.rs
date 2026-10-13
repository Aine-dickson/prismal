//! Units: named scale factors relative to coherent SI units (MK-3.6, MK-3.11).

use crate::dim::{Dim, Ratio};
use serde::{Deserialize, Serialize};

/// A unit as written by the author, with its meaning in coherent SI.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    /// What the author wrote; display only (MK-3.7).
    pub text: String,
    pub scale: f64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub offset: f64,
    pub dim: Dim,
}

fn is_zero(x: &f64) -> bool {
    *x == 0.0
}

/// Symbol, scale to coherent SI, dimension.
fn lookup(sym: &str) -> Option<(f64, Dim)> {
    let l = Dim::length();
    let m = Dim::mass();
    let t = Dim::time();
    let force = m.mul(&l).div(&t.pow(Ratio::int(2)));
    let energy = force.mul(&l);
    Some(match sym {
        "m" => (1.0, l),
        "cm" => (0.01, l),
        "mm" => (0.001, l),
        "km" => (1000.0, l),
        "s" => (1.0, t),
        "ms" => (0.001, t),
        "min" => (60.0, t),
        "h" => (3600.0, t),
        "kg" => (1.0, m),
        "g" => (0.001, m),
        "N" => (1.0, force),
        "J" => (1.0, energy),
        "W" => (1.0, energy.div(&t)),
        "Pa" => (1.0, force.div(&l.pow(Ratio::int(2)))),
        "Hz" => (1.0, t.inv()),
        "A" => (1.0, Dim::base(3)),
        "K" => (1.0, Dim::base(4)),
        "mol" => (1.0, Dim::base(5)),
        "cd" => (1.0, Dim::base(6)),
        // Plane angle is dimensionless (D-021).
        "rad" => (1.0, Dim::NONE),
        "deg" => (std::f64::consts::PI / 180.0, Dim::NONE),
        "rev" => (2.0 * std::f64::consts::PI, Dim::NONE),
        // A hundredth: `25 %` is 0.25; `unit %` shows a share as a percentage (D-077).
        "%" => (0.01, Dim::NONE),
        _ => return None,
    })
}

impl Unit {
    /// True if `sym` is a unit symbol this implementation knows.
    pub fn is_symbol(sym: &str) -> bool {
        lookup(sym).is_some()
    }

    /// Parses a unit expression: `m/s^2`, `N/m`, `/m`, `kg*m^2`, `deg`; exponents are whole
    /// or half numbers, `/s^0.5` (D-090: the coefficients of white noise).
    pub fn parse(text: &str) -> Result<Unit, String> {
        let mut scale = 1.0;
        let mut dim = Dim::NONE;
        let mut sign = 1;
        let spaced = text.replace('*', " ").replace('/', " / ");
        for tok in spaced.split_whitespace() {
            if tok == "/" {
                sign = -1;
                continue;
            }
            // Twice the exponent, a whole number.
            let (sym, twice) = match tok.split_once('^') {
                Some((a, b)) => match b.parse::<f64>() {
                    Ok(x) if (2.0 * x).fract() == 0.0 && x.abs() < 1e6 => (a, (2.0 * x) as i32),
                    _ => return Err(format!("bad unit exponent in `{text}`: a whole or half number")),
                },
                None => (tok, 2),
            };
            let (s, d) = lookup(sym).ok_or_else(|| format!("unknown unit `{sym}`"))?;
            let e = sign * twice;
            scale *= s.powf(e as f64 / 2.0);
            dim = dim.mul(&d.pow(Ratio::new(e, 2)));
        }
        // Every factor after the first `/` is in the denominator: `kg/m/s` is kg m^-1 s^-1.
        Ok(Unit { text: text.to_string(), scale, offset: 0.0, dim })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_units() {
        let u = Unit::parse("m/s^2").unwrap();
        assert_eq!(u.dim, Dim::parse("L T^-2").unwrap());
        assert_eq!(u.scale, 1.0);
        let k = Unit::parse("/m").unwrap();
        assert_eq!(k.dim, Dim::parse("1/L").unwrap());
        let n = Unit::parse("N/m").unwrap();
        assert_eq!(n.dim, Dim::parse("M T^-2").unwrap());
        let h = Unit::parse("/s^0.5").unwrap();
        assert_eq!(h.dim, Dim::time().pow(Ratio::new(-1, 2)));
        assert!(Unit::parse("s^0.3").is_err());
        let d = Unit::parse("deg").unwrap();
        assert!(d.dim.is_none());
        assert_eq!(10.0 * d.scale, 10.0 * (std::f64::consts::PI / 180.0));
    }
}
