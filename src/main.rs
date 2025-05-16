use std::{io::{self, stderr, stdin, stdout, BufRead, StdinLock, StdoutLock, Write}, process::exit};
use clap::{Parser, Arg, ArgAction};

use zh_num::{
    parser::{hard_number, number},
    Number, ZhNum, ZhNumUpper,
};

const CRLF: &str = "\r\n";
const LF: &str = "\n";

fn get_eol(s: &str) -> &str {
    s.ends_with(CRLF)
        .then_some(CRLF)
        .or_else(|| s.ends_with(LF)
            .then_some(LF))
        .unwrap_or_default()
}

#[derive(Debug, Default, Parser)]
#[command(help_template = "\
{usage-heading} {usage}
{about}

{before-help}{all-args}{after-help}

{name}@{version}
{author}
")]
#[command(
    about = "将ASCII数字和中文数字相互转换",
    version,
    author,
    disable_version_flag = true,
    arg = Arg::new("version-lower")
        .short('v')
        .long("version")
        .help("Print version")
        .action(ArgAction::Version),
)]
struct Config {
    #[arg(short, help = "反向转换, 也就是将ASCII数字转换成中文数字")]
    dump: bool,
    #[arg(short = 'D', help = "类似 -d, 但是中文数字是大写")]
    is_upper: bool,
    #[arg(short, help = "转换时保留结果之外的文本")]
    rem: bool,
    #[arg(short = 'a', help = "转换硬数字, 如 `千零二三` `一零零十三`")]
    hard: bool,
    #[arg(short, help = "识别时跳过一部分字符, 如果给定了-r则会留在结果中")]
    #[arg(default_value_t = 0)]
    skip_ch: usize,
}
impl Config {
    fn num_fmt(&self) -> fn(&mut io::StdoutLock, Number) -> io::Result<()> {
        if !self.is_upper {
            |f, n| write!(f, "{}", ZhNum(n))
        } else {
            |f, n| write!(f, "{}", ZhNumUpper(n))
        }
    }

    fn init_dependenices(mut self) -> Self {
        if self.dump && self.hard { eprintln!("警告: 在指定 -d 时 -a 被忽略"); }
        if self.is_upper && self.hard { eprintln!("警告: 在指定 -D 时 -a 被忽略"); }
        self.dump |= self.is_upper;
        self
    }
}

struct Processor {
    lnum: u64,
    noeol: bool,
    cfg: Config,
    line: String,
    stdin: StdinLock<'static>,
    stdout: StdoutLock<'static>,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            lnum: Default::default(),
            noeol: Default::default(),
            cfg: Default::default(),
            line: Default::default(),
            stdin: stdin().lock(),
            stdout: stdout().lock(),
        }
    }
}

impl Processor {
    fn skip_ch_line<'a>(&self, line: &'a str) -> (&'a str, &'a str) {
        line.char_indices()
            .nth(self.cfg.skip_ch)
            .map_or((line, ""), |(i, _)| {
                let s = line[i..].len() - line[i..].trim_start().len();
                line.split_at(i+s)
            })
    }

    fn fetch_line(&mut self) -> io::Result<usize> {
        if !self.noeol {
            self.lnum += 1;
        }
        self.line.clear();

        let bytec = self.stdin.read_line(&mut self.line)?;
        self.noeol = get_eol(&self.line).is_empty();
        Ok(bytec)
    }

    fn run_dump_lines(&mut self) -> io::Result<()> {
        while self.fetch_line()? != 0 {
            let (prefix, line) = self.skip_ch_line(&self.line);
            let rem_idx = line
                .find(|ch| !char::is_ascii_digit(&ch))
                .unwrap_or(line.len());
            let (part, rem_str) = line.split_at(rem_idx);
            let num = part
                .parse()
                .map(Some)
                .or_else(|e| {
                    writeln!(
                        stderr(),
                        "`{part}` ({}) {}: {e}",
                        rem_str.trim_end(),
                        self.lnum,
                    )?;
                    io::Result::Ok(None)
                })?;

            if self.cfg.rem {
                write!(self.stdout, "{prefix}")?;
            }
            if let Some(num) = num {
                self.cfg.num_fmt()(&mut self.stdout, num)?;
            } else {
                write!(self.stdout, "{part}")?;
            }
            if self.cfg.rem {
                write!(self.stdout, "{rem_str}")?;
            } else {
                write!(self.stdout, "{}", get_eol(rem_str))?;
            }
        }
        Ok(())
    }

    fn run_make_lines(&mut self) -> io::Result<()> {
        while self.fetch_line()? != 0 {
            let (prefix, line) = self.skip_ch_line(&self.line);
            let result = if !self.cfg.hard {
                number(line)
            } else {
                hard_number(line)
            };
            let (n, rem_str) = result
                .map(|(n, s)| (Some(n), s))
                .or_else(|e| {
                    writeln!(stderr(), "`{}` {}:{} expected {}",
                        line.trim_end(),
                        self.lnum,
                        e.location.column+self.cfg.skip_ch,
                        e.expected,
                    )?;
                    io::Result::Ok((None, line))
                })?;
            if self.cfg.rem {
                write!(self.stdout, "{prefix}")?;
            }
            if let Some(n) = n {
                write!(self.stdout, "{n}")?;
            }
            if self.cfg.rem {
                write!(self.stdout, "{rem_str}")?;
            } else {
                write!(self.stdout, "{}", get_eol(rem_str))?;
            }
        }
        Ok(())
    }

    fn run_lines(&mut self) -> io::Result<()> {
        if self.cfg.dump {
            self.run_dump_lines()
        } else {
            self.run_make_lines()
        }
    }
}

fn main() {
    let cfg = Config::parse().init_dependenices();
    let mut processor = Processor {
        cfg,
        ..Default::default()
    };

    match processor.run_lines() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Error at line {}: {e}", processor.lnum);
            exit(1)
        },
    }
}
