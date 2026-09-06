//! The parameter registry's schema, its dimension algebra, and the evaluator for a `derived`
//! expression (§16.1).
//!
//! **The registry is data and this is its reader.** In M0 there is no engine type behind it: §1
//! forbids engine code in this milestone, and what matters is that the rules are mechanical before
//! anything can break them, which a checker over data achieves as completely as a type would. The
//! engine-side `Entry` and a unit vocabulary generated from `domain`'s quantity types land in M1.

use std::collections::BTreeMap;
use std::path::Path;

/// A dimension: base dimensions to exponents. Dimensionless is the empty map, which is `ratio`.
pub type Dimension = BTreeMap<String, i32>;

/// One registry entry, as read from the file. Validation is the checker's, not the parser's.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Dotted, unique.
    pub name: String,
    /// `model` — a claim about the world, counted against M3. `capacity` — an engineering size.
    pub namespace: String,
    /// The number.
    pub value: f64,
    /// Drawn from the closed vocabulary; the unit determines the dimension.
    pub unit: String,
    /// `world`, `axis:k`, or `region:r`.
    pub scope: String,
    /// The system that reads it.
    pub owner: String,
    /// `structural` | `derived` | `assumed` | `placeholder`.
    pub provenance: String,
    /// `structural`: the definitional identity it names.
    pub identity: Option<String>,
    /// `derived`: the expression it is computed by.
    pub expression: Option<String>,
    /// `assumed`: the bracket its value must lie inside.
    pub bracket: Option<(f64, f64)>,
    /// `assumed`: the asymmetry axis it attaches to, or `none`.
    pub axis: Option<String>,
    /// In model terms only.
    pub justification: String,
}

/// The closed unit vocabulary and which units an `assumed` entry may carry (§16.1 rule 1).
pub struct Units {
    /// unit name to its base dimension.
    pub dimension: BTreeMap<String, Dimension>,
    /// The five §16.1 rule 1 admits for `assumed`.
    pub assumable: Vec<String>,
}

impl Units {
    /// Read `registry/units.txt`.
    ///
    /// # Errors
    /// If the file is unreadable, or a line does not carry a name, a dimension and an assumable flag.
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = root.join("registry/units.txt");
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut dimension = BTreeMap::new();
        let mut assumable = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut f = line.split_whitespace();
            let (Some(name), Some(dim), Some(ass)) = (f.next(), f.next(), f.next()) else {
                return Err(format!("units.txt: cannot read `{line}`"));
            };
            let mut d = Dimension::new();
            if dim != "-" {
                d.insert(dim.to_owned(), 1);
            }
            dimension.insert(name.to_owned(), d);
            if ass == "yes" {
                assumable.push(name.to_owned());
            }
        }
        Ok(Self {
            dimension,
            assumable,
        })
    }

    /// The dimension of a unit, which may be compound: `minor-unit/hour`.
    pub fn dimension_of(&self, unit: &str) -> Option<Dimension> {
        if let Some((num, den)) = unit.split_once('/') {
            let mut d = self.dimension.get(num)?.clone();
            for (k, v) in self.dimension.get(den)? {
                *d.entry(k.clone()).or_insert(0) -= v;
            }
            d.retain(|_, v| *v != 0);
            return Some(d);
        }
        self.dimension.get(unit).cloned()
    }
}

/// The sixteen definitional identities (§16.1 rule 4, ADR-0013).
///
/// # Errors
/// If `registry/identities.txt` is unreadable.
pub fn identities(root: &Path) -> Result<Vec<String>, String> {
    let path = root.join("registry/identities.txt");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_whitespace().next().map(str::to_owned))
        .collect())
}

/// Read the entries file.
///
/// # Errors
/// If the file is unreadable, does not parse, or an entry is missing a required field.
pub fn entries(root: &Path) -> Result<Vec<Entry>, String> {
    let path = root.join("registry/entries.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_entries(&text)
}

/// Parse an entries document. Split from `entries` so a fixture can be parsed from a string.
///
/// # Errors
/// If the document does not parse, has no `[[entry]]` array, or an entry lacks a required field.
pub fn parse_entries(text: &str) -> Result<Vec<Entry>, String> {
    let table: toml::Table = text.parse().map_err(|e| format!("does not parse: {e}"))?;
    let rows = table
        .get("entry")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "no [[entry]] array".to_owned())?;
    let mut out = Vec::new();
    for row in rows {
        let s = |k: &str| row.get(k).and_then(toml::Value::as_str).map(str::to_owned);
        let name = s("name").ok_or_else(|| "an entry has no `name`".to_owned())?;
        out.push(Entry {
            namespace: s("namespace").unwrap_or_else(|| "model".to_owned()),
            value: row
                .get("value")
                .and_then(toml::Value::as_float)
                .ok_or_else(|| format!("{name}: no numeric `value`"))?,
            unit: s("unit").ok_or_else(|| format!("{name}: no `unit`"))?,
            scope: s("scope").ok_or_else(|| format!("{name}: no `scope`"))?,
            owner: s("owner").ok_or_else(|| format!("{name}: no `owner`"))?,
            provenance: s("provenance").ok_or_else(|| format!("{name}: no `provenance`"))?,
            identity: s("identity"),
            expression: s("expression"),
            bracket: row
                .get("bracket")
                .and_then(toml::Value::as_array)
                .and_then(|b| {
                    let lo = b.first()?.as_float()?;
                    let hi = b.get(1)?.as_float()?;
                    Some((lo, hi))
                }),
            axis: s("axis"),
            justification: s("justification")
                .ok_or_else(|| format!("{name}: no `justification`"))?,
            name,
        });
    }
    Ok(out)
}

/// What an expression evaluated to: its value and its dimension.
pub struct Evaluated {
    /// The number.
    pub value: f64,
    /// Its dimension, derived from the units of what it referenced.
    pub dimension: Dimension,
}

/// Evaluate a `derived` expression against the other entries, carrying dimensions through.
///
/// The grammar is deliberately tiny: entry names, the four operators, parentheses, and literals
/// drawn from `{0, 1, -1, 2}` (§16.1 rule 6). **Anything else is a value wearing an expression's
/// clothes**, which is the whole reason the literal set is closed.
///
/// # Errors
/// If the expression does not lex or parse, names an entry that does not exist, uses a literal outside
/// the closed set, or combines dimensions that do not agree.
pub fn evaluate(
    expr: &str,
    by_name: &BTreeMap<String, &Entry>,
    units: &Units,
) -> Result<Evaluated, String> {
    let tokens = lex(expr)?;
    let mut p = Parser {
        tokens,
        at: 0,
        by_name,
        units,
    };
    let v = p.expression()?;
    if p.at < p.tokens.len() {
        return Err(format!("trailing tokens after position {}", p.at));
    }
    Ok(v)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Name(String),
    Number(f64),
    Op(char),
    Open,
    Close,
}

/// The four literals §16.1 rule 6 admits.
const LITERALS: [f64; 4] = [0.0, 1.0, -1.0, 2.0];

fn lex(expr: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars.get(i).copied().unwrap_or(' ');
        if c.is_whitespace() {
            i += 1;
        } else if c == '(' {
            out.push(Token::Open);
            i += 1;
        } else if c == ')' {
            out.push(Token::Close);
            i += 1;
        } else if "+-*/".contains(c) {
            out.push(Token::Op(c));
            i += 1;
        } else if c.is_ascii_digit() {
            let start = i;
            while i < chars.len()
                && chars
                    .get(i)
                    .is_some_and(|c| c.is_ascii_digit() || *c == '.')
            {
                i += 1;
            }
            let s: String = chars.get(start..i).unwrap_or_default().iter().collect();
            let n: f64 = s.parse().map_err(|_| format!("`{s}` is not a number"))?;
            if !LITERALS.contains(&n) {
                return Err(format!(
                    "literal `{s}` is not one of 0, 1, -1, 2 (§16.1 rule 6) — a literal outside that \
                     set is a value wearing an expression's clothes"
                ));
            }
            out.push(Token::Number(n));
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len()
                && chars
                    .get(i)
                    .is_some_and(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
            {
                i += 1;
            }
            out.push(Token::Name(
                chars.get(start..i).unwrap_or_default().iter().collect(),
            ));
        } else {
            return Err(format!("unexpected character `{c}`"));
        }
    }
    Ok(out)
}

struct Parser<'a> {
    tokens: Vec<Token>,
    at: usize,
    by_name: &'a BTreeMap<String, &'a Entry>,
    units: &'a Units,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn expression(&mut self) -> Result<Evaluated, String> {
        let mut left = self.term()?;
        while let Some(Token::Op(op @ ('+' | '-'))) = self.peek().cloned() {
            self.at += 1;
            let right = self.term()?;
            if left.dimension != right.dimension {
                return Err(format!(
                    "dimensions disagree across `{op}`: {} against {}",
                    show(&left.dimension),
                    show(&right.dimension)
                ));
            }
            left.value = if op == '+' {
                left.value + right.value
            } else {
                left.value - right.value
            };
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<Evaluated, String> {
        let mut left = self.atom()?;
        while let Some(Token::Op(op @ ('*' | '/'))) = self.peek().cloned() {
            self.at += 1;
            let right = self.atom()?;
            let mut d = left.dimension.clone();
            for (k, v) in &right.dimension {
                let e = d.entry(k.clone()).or_insert(0);
                *e += if op == '*' { *v } else { -*v };
            }
            d.retain(|_, v| *v != 0);
            left.value = if op == '*' {
                left.value * right.value
            } else {
                left.value / right.value
            };
            left.dimension = d;
        }
        Ok(left)
    }

    fn atom(&mut self) -> Result<Evaluated, String> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.at += 1;
                Ok(Evaluated {
                    value: n,
                    dimension: Dimension::new(),
                })
            }
            Some(Token::Name(n)) => {
                self.at += 1;
                let e = self
                    .by_name
                    .get(&n)
                    .ok_or_else(|| format!("`{n}` names no entry"))?;
                let dimension = self.units.dimension_of(&e.unit).ok_or_else(|| {
                    format!(
                        "`{n}` has unit `{}`, which is outside the vocabulary",
                        e.unit
                    )
                })?;
                Ok(Evaluated {
                    value: e.value,
                    dimension,
                })
            }
            Some(Token::Open) => {
                self.at += 1;
                let v = self.expression()?;
                if self.peek() != Some(&Token::Close) {
                    return Err("unclosed parenthesis".to_owned());
                }
                self.at += 1;
                Ok(v)
            }
            other => Err(format!("expected a value, found {other:?}")),
        }
    }
}

/// A dimension, printed the way an error message should show it.
pub fn show(d: &Dimension) -> String {
    if d.is_empty() {
        return "dimensionless".to_owned();
    }
    d.iter()
        .map(|(k, v)| {
            if *v == 1 {
                k.clone()
            } else {
                format!("{k}^{v}")
            }
        })
        .collect::<Vec<_>>()
        .join("·")
}
