use clap::CommandFactory;
use clap::Parser;
use colored::Colorize;
use difference::{Changeset, Difference};
use regex::{Captures, Regex, Replacer};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// replace newline with space
    #[arg(short, long)]
    lines: bool,
    /// do not detect/replace latex commands
    #[arg(short, long)]
    keep_latex: bool,
    // /// deletion
    // #[arg(short, long, value_name = "PATTERN", default_value = "#rem[\\1]")]
    // deletion: String,
    // /// addition
    // #[arg(short, long, value_name = "PATTERN", default_value = "#add[\\1]")]
    // addition: String,
    /// old file
    old_file: Option<PathBuf>,
    /// new file
    new_file: Option<PathBuf>,
    /// aux file
    aux_file: Option<PathBuf>,
}

fn main() {
    let args = Cli::parse();
    match (args.old_file, args.new_file, args.aux_file) {
        (Some(old), Some(new), aux) => {
            let old = std::fs::read_to_string(old).unwrap();
            let new = std::fs::read_to_string(new).unwrap();
            let old_map = text_labels(&old);
            let new_map = text_labels(&new);
            let mut buf = String::with_capacity(200);
            let (citations, references) = if let Some(aux) = aux {
                let aux_contents = std::fs::read_to_string(aux).unwrap();
                (cite_label(&aux_contents), ref_label(&aux_contents))
            } else {
                (HashMap::new(), HashMap::new())
            };
            // println!("{citations:#?}");
            let pat = LatexCmd::pattern();
            loop {
                println!("** Label:");
                buf.clear();
                std::io::stdin().read_line(&mut buf).unwrap();
                let label = buf.trim();
                let difftext = match (old_map.get(label), new_map.get(label)) {
                    (Some(o), Some(n)) => get_diff(o, n, args.lines),
                    (Some(o), None) => format!(
                        "#rem[{}]",
                        if args.lines {
                            o.replace("\n", " ").blue()
                        } else {
                            o.blue()
                        }
                    ),
                    (None, Some(n)) => format!(
                        "#add[{}]",
                        if args.lines {
                            n.replace("\n", " ").blue()
                        } else {
                            n.blue()
                        }
                    ),
                    (None, None) => {
                        println!("Label not found in both text");
                        continue;
                    }
                };

                if args.keep_latex {
                    println!("{}", difftext)
                } else {
                    println!(
                        "{}",
                        pat.replace_all(&difftext, LatexCmd::new(&citations, &references))
                    )
                }
            }
        }
        (None, None, None) => repl(args.lines),
        _ => {
            _ = Cli::command().print_help();
        }
    }
}

struct LatexCmd<'a> {
    cite_map: &'a HashMap<String, (String, String)>,
    ref_map: &'a HashMap<String, String>,
}

impl<'a> LatexCmd<'a> {
    fn new(
        cite_map: &'a HashMap<String, (String, String)>,
        ref_map: &'a HashMap<String, String>,
    ) -> Self {
        Self { cite_map, ref_map }
    }
    fn pattern() -> Regex {
        regex::Regex::new("\\\\(?<t>\\w+)\\{(?<entry>.+?)\\}").unwrap()
    }
}

impl<'a> Replacer for LatexCmd<'a> {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        let entry: &str = &caps["entry"];
        match &caps["t"] {
            "ref" => {
                let text = self.ref_map.get(entry).map(|n| n.as_str()).unwrap_or(entry);
                dst.push_str(text);
            }
            "cite" => {
                let text = entry
                    .split(',')
                    .map(|c| {
                        self.cite_map
                            .get(c)
                            .map(|(a, y)| format!("{a} ({y})"))
                            .unwrap_or(c.to_string())
                    })
                    .collect::<Vec<String>>()
                    .join(", ");
                dst.push_str(&format!("({text})"));
            }
            "citep" => {
                let text = entry
                    .split(',')
                    .map(|c| {
                        self.cite_map
                            .get(c)
                            .map(|(a, y)| format!("{a}, {y}"))
                            .unwrap_or(c.to_string())
                    })
                    .collect::<Vec<String>>()
                    .join("; ");
                dst.push_str(&format!("({text})"));
            }
            "texttt" => {
                dst.push_str(&format!("`{entry}`"));
            }
            "url" => {
                dst.push_str(&format!("#link(\"{entry}\")"));
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

fn ref_label(txt: &str) -> HashMap<String, String> {
    // \newlabel{fig:ohio-map}{{7}{15}...
    txt.lines()
        .filter_map(|l| l.strip_prefix("\\newlabel{"))
        .filter_map(|l| {
            let mut data = l.split("}{");
            let label = data
                .next()?
                .trim_start_matches('{')
                .trim_end_matches('}')
                .to_string();
            let number = data
                .next()?
                .trim_start_matches('{')
                .trim_end_matches('}')
                .to_string();
            Some((label, number))
        })
        .collect()
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
    println!("Running in interactive mode");
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
