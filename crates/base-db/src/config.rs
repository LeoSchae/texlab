use std::time::Duration;

use parser::SyntaxConfig;
use regex::Regex;

#[derive(Debug)]
pub struct Config {
    pub root_dir: Option<String>,
    pub build: BuildConfig,
    pub diagnostics: DiagnosticsConfig,
    pub formatting: FormattingConfig,
    pub synctex: Option<SynctexConfig>,
    pub symbols: SymbolConfig,
    pub syntax: SyntaxConfig,
    pub completion: CompletionConfig,
}

#[derive(Debug)]
pub struct BuildConfig {
    pub program: String,
    pub args: Vec<String>,
    pub on_save: bool,
    pub forward_search_after: bool,
    pub output_dir: String,
}

#[derive(Debug)]
pub struct DiagnosticsConfig {
    pub allowed_patterns: Vec<Regex>,
    pub ignored_patterns: Vec<Regex>,
    pub chktex: ChktexConfig,
    pub delay: Duration,
}

#[derive(Debug)]
pub struct ChktexConfig {
    pub on_open: bool,
    pub on_save: bool,
    pub on_edit: bool,
}

#[derive(Debug)]
pub struct SynctexConfig {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct FormattingConfig {
    pub tex_formatter: Formatter,
    pub bib_formatter: Formatter,
    pub latex_indent: LatexIndentConfig,
    pub line_length: usize,
}

#[derive(Debug)]
pub enum Formatter {
    Null,
    Server,
    LatexIndent,
}

#[derive(Debug)]
pub struct LatexIndentConfig {
    pub local: Option<String>,
    pub modify_line_breaks: bool,
}

#[derive(Debug)]
pub struct SymbolConfig {
    pub allowed_patterns: Vec<Regex>,
    pub ignored_patterns: Vec<Regex>,
}

#[derive(Debug)]
pub struct CompletionConfig {
    pub matcher: MatchingAlgo,
}

#[derive(Debug)]
pub enum MatchingAlgo {
    Skim,
    SkimIgnoreCase,
    Prefix,
    PrefixIgnoreCase,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_dir: None,
            build: BuildConfig::default(),
            diagnostics: DiagnosticsConfig::default(),
            formatting: FormattingConfig::default(),
            synctex: None,
            symbols: SymbolConfig::default(),
            syntax: SyntaxConfig::default(),
            completion: CompletionConfig::default(),
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            program: String::from("latexmk"),
            args: ["-pdf", "-interaction=nonstopmode", "-synctex=1", "%f"]
                .into_iter()
                .map(String::from)
                .collect(),
            on_save: false,
            forward_search_after: false,
            output_dir: String::from("."),
        }
    }
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            allowed_patterns: Vec::new(),
            ignored_patterns: Vec::new(),
            delay: Duration::from_millis(300),
            chktex: ChktexConfig::default(),
        }
    }
}

impl Default for ChktexConfig {
    fn default() -> Self {
        Self {
            on_open: false,
            on_save: false,
            on_edit: false,
        }
    }
}

impl Default for FormattingConfig {
    fn default() -> Self {
        Self {
            tex_formatter: Formatter::LatexIndent,
            bib_formatter: Formatter::Server,
            line_length: 80,
            latex_indent: LatexIndentConfig::default(),
        }
    }
}

impl Default for LatexIndentConfig {
    fn default() -> Self {
        Self {
            local: None,
            modify_line_breaks: false,
        }
    }
}

impl Default for SymbolConfig {
    fn default() -> Self {
        Self {
            allowed_patterns: Vec::new(),
            ignored_patterns: Vec::new(),
        }
    }
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            matcher: MatchingAlgo::SkimIgnoreCase,
        }
    }
}
