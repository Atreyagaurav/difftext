use colored::Colorize;
use difference::{Changeset, Difference};
use regex::{Captures, Regex, Replacer};
use std::collections::HashMap;

struct LatexCmd<'a> {
    map: &'a HashMap<String, (String, String)>,
}

impl<'a> LatexCmd<'a> {
    fn new(map: &'a HashMap<String, (String, String)>) -> Self {
        Self { map }
    }
    fn pattern() -> Regex {
        regex::Regex::new("\\\\(?<t>\\w+)\\{(?<entry>.+?)\\}").unwrap()
    }
}

impl<'a> Replacer for LatexCmd<'a> {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        let entry: &str = &caps["entry"];
        match &caps["t"] {
            "cite" => {
                let text = entry
                    .split(',')
                    .filter_map(|c| self.map.get(c))
                    .map(|(a, y)| format!("{a} ({y})"))
                    .collect::<Vec<String>>()
                    .join(", ");
                dst.push_str(&format!("({text})"));
            }
            "citep" => {
                let text = entry
                    .split(',')
                    .filter_map(|c| self.map.get(c))
                    .map(|(a, y)| format!("{a}, {y}"))
                    .collect::<Vec<String>>()
                    .join("; ");
                dst.push_str(&format!("({text})"));
            }
            "texttt" => {
                dst.push_str(&format!("`{entry}`"));
            }
            cmd => {
                // unknown commands will be translated as typst
                // commands, can just add functions in typst to make
                // it work
                dst.push_str(&format!("#{cmd}()[{entry}]"));
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_, old, new, aux @ ..] => {
            let old = std::fs::read_to_string(old).unwrap();
            let new = std::fs::read_to_string(new).unwrap();
            let old_map = text_labels(&old);
            let new_map = text_labels(&new);
            let mut buf = String::with_capacity(200);
            let citations = if let Some(aux) = aux.get(0) {
                cite_label(&std::fs::read_to_string(aux).unwrap())
            } else {
                HashMap::new()
            };
            let lines = aux.contains(&"-l".to_string());
            // println!("{citations:#?}");
            let pat = LatexCmd::pattern();
            loop {
                println!("** Label:");
                buf.clear();
                std::io::stdin().read_line(&mut buf).unwrap();
                let label = buf.trim();
                match (old_map.get(label), new_map.get(label)) {
                    (Some(o), Some(n)) => {
                        println!(
                            "{}",
                            pat.replace_all(&get_diff(o, n, lines), LatexCmd::new(&citations))
                        )
                    }
                    (Some(_), None) => println!("Label not found in new text"),
                    (None, Some(_)) => println!("Label not found in old text"),
                    (None, None) => println!("Label not found in both text"),
                }
            }
        }
        [_, rest @ ..] => {
            let lines = rest.contains(&"-l".to_string());
            repl(lines)
        }
        _ => print_help(),
    }
}

fn print_help() {
    println!("Usage: difftext [old] [new] [aux]")
}

fn cite_label(txt: &str) -> HashMap<String, (String, String)> {
    // Each entry are like the one below, one in each line
    // \bibcite{zhangUnderstandingRuntimePerformance2023}{{73}{2023}{{Zhang et~al.\spacefactor \@m {}}}{{}}}
    txt.lines()
        .filter_map(|l| l.strip_prefix("\\bibcite{"))
        .filter_map(|l| {
            let mut data = l.split("}{");
            let key = data
                .next()?
                .trim_start_matches('{')
                .trim_end_matches('}')
                .to_string();
            data.next();
            let year = data
                .next()?
                .trim_start_matches('{')
                .trim_end_matches('}')
                .to_string();
            let auth = data
                .next()?
                .split("\\spacefactor")
                .next()?
                .trim_start_matches('{')
                .trim_end_matches('}')
                .replace('~', " ")
                .to_string();
            Some((key, (auth, year)))
        })
        .collect()
}

fn text_labels(txt: &str) -> HashMap<String, &str> {
    // \paralabel{par:intro-gis}
    let labels: Vec<(&str, &str)> = txt
        .split("\\paralabel{par:")
        .skip(1)
        .map(|p| p.split("\n\n").next().unwrap().trim())
        .filter_map(|p| p.split_once('}'))
        .collect();
    let labelmap = labels
        .into_iter()
        .enumerate()
        .map(|(i, (l, t))| [((i + 1).to_string(), t), (l.to_string(), t)])
        .flatten()
        .collect();

    labelmap
}

fn repl(lines: bool) {
    loop {
        println!("** Old text:");
        let old = std::io::read_to_string(std::io::stdin())
            .unwrap()
            .replace("\n", " ");
        println!("** New text:");
        let new = std::io::read_to_string(std::io::stdin())
            .unwrap()
            .replace("\n", " ");
        println!("{}", get_diff(&old, &new, lines));
    }
}

fn get_diff(old: &str, new: &str, lines: bool) -> String {
    let changes = if lines {
        Changeset::new(&old.replace('\n', " "), &new.replace('\n', " "), " ")
    } else {
        Changeset::new(old, new, " ")
    };
    println!("** Changes:\n");
    let mut diff = String::with_capacity(old.len() + new.len());
    for ch in changes.diffs {
        match ch {
            Difference::Same(txt) => diff.push_str(&txt),
            Difference::Add(txt) => diff.push_str(&format!("#add[{}]", txt.blue())),
            Difference::Rem(txt) => diff.push_str(&format!("#rem[{}]", txt.red())),
        }
        diff.push(' ');
    }
    diff
}
