//! # `adr new` — the counter, so that a number is allocated rather than chosen
//!
//! W5.1. Two people picking the next free number by looking at the directory will eventually pick the
//! same one, and the duplicate is invisible until someone reads both. Allocating from a committed file
//! moves the collision to where a tool can see it: both branches append to the same line of
//! `decisions/register.txt`, git refuses the merge, and the fix is one line.
//!
//! **A stub starts red.** `guard`, `cost` and `alternatives-rejected` are written empty, so
//! `check-adr` fails until they are filled. Appendix A's rule is that a decision which cannot be given
//! a mechanical guard may not be entered; a stub that passed the gate would make that rule optional
//! for exactly as long as it took to forget.

use std::path::Path;
use std::process::ExitCode;

use crate::check_adr::read_register;

/// `adr new "<title>"` allocates the next number; `adr new <NNNN>` writes a reserved one.
pub fn run(root: &Path, arg: Option<&str>) -> ExitCode {
    let Some(arg) = arg else {
        eprintln!("usage: aurora-tools adr new <\"title\"|NNNN>");
        return ExitCode::FAILURE;
    };
    let Some(register) = read_register(root) else {
        eprintln!("decisions/register.txt is missing — there is nothing to allocate from");
        return ExitCode::FAILURE;
    };

    // A bare number claims a reservation; anything else is a new title.
    let (id, slug, title, allocated) = if let Ok(id) = arg.trim().parse::<u16>() {
        let Some((_, slug)) = register.iter().find(|(r, _)| *r == id) else {
            eprintln!("{id:04} is not allocated in decisions/register.txt");
            return ExitCode::FAILURE;
        };
        (id, slug.clone(), title_from(slug), false)
    } else {
        let slug = slugify(arg);
        if slug.is_empty() {
            eprintln!("a title must contain something that slugifies");
            return ExitCode::FAILURE;
        }
        let next = register.iter().map(|(id, _)| *id).max().unwrap_or(0) + 1;
        (next, slug, arg.to_owned(), true)
    };

    let path = root.join(format!("decisions/ADR-{id:04}-{slug}.md"));
    if path.exists() {
        eprintln!("{} already exists", path.display());
        return ExitCode::FAILURE;
    }

    if allocated {
        let line = format!("{id:04} {slug}\n");
        let register_path = root.join("decisions/register.txt");
        let Ok(mut text) = std::fs::read_to_string(&register_path) else {
            eprintln!("cannot read {}", register_path.display());
            return ExitCode::FAILURE;
        };
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&line);
        if std::fs::write(&register_path, text).is_err() {
            eprintln!("cannot write {}", register_path.display());
            return ExitCode::FAILURE;
        }
    }

    if std::fs::write(&path, stub(id, &title)).is_err() {
        eprintln!("cannot write {}", path.display());
        return ExitCode::FAILURE;
    }
    println!("{}", path.display());
    println!(
        "  {} — `guard`, `cost` and `alternatives-rejected` are empty, so `check-adr` fails until\n  \
         they are filled. That is the stub working.",
        if allocated {
            "allocated"
        } else {
            "reservation written"
        }
    );
    ExitCode::SUCCESS
}

/// Lowercase, and every run of characters that is not a letter or digit becomes one hyphen.
#[must_use]
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.extend(ch.to_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

/// A slug read back as a title: hyphens to spaces, first letter raised. Editable, and meant to be.
fn title_from(slug: &str) -> String {
    let spaced = slug.replace('-', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => spaced,
    }
}

/// The eleven fields, three of them empty on purpose.
fn stub(id: u16, title: &str) -> String {
    format!(
        "---\n\
         id: ADR-{id:04}\n\
         title: {title}\n\
         status: proposed\n\
         date: \n\
         register-entry: \n\
         claim-impact: \n\
         guard: \n\
         supersedes: none\n\
         cost: \n\
         alternatives-rejected: \n\
         re-derivations: none\n\
         ---\n\
         \n\
         ## Decision\n\
         \n\
         ## Why\n\
         \n\
         ## What it costs\n\
         \n\
         ## Alternatives rejected\n\
         \n\
         ## The guard\n\
         \n\
         <!-- Appendix A: a decision that cannot be given a named mechanical guard may not be\n\
         entered. Name the check, and what it refuses. -->\n"
    )
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn slugify_collapses_and_trims() {
        assert_eq!(
            slugify("The burn-in multiplicity correction"),
            "the-burn-in-multiplicity-correction"
        );
        assert_eq!(
            slugify("  i64 conserved quantities: overflow panics  "),
            "i64-conserved-quantities-overflow-panics"
        );
        assert_eq!(slugify("§7.5 — 44 B or 148 B"), "7-5-44-b-or-148-b");
        assert_eq!(slugify("---"), "");
    }
}
