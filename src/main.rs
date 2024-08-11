use std::io::{self, stderr, stdin, stdout, Write, BufRead};
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
        self.dump |= self.is_upper;
        self
    }
}

fn main() -> io::Result<()> {
    let cfg = Config::parse().init_dependenices();
    let Config { rem, skip_ch, dump, hard, .. } = cfg;
    macro_rules! skip_ch_line {
        ($line:expr) => {{
            fn convf<'a, T, F>(f: F) -> F
            where T: ?Sized + 'a,
                  F: FnOnce(&'a T) -> (&'a T, &'a T),
            {
                f
            }
            convf(|line: &str| {
                line.char_indices()
                    .nth(skip_ch)
                    .map(|(i, _)| {
                        let s = line[i..].len() - line[i..].trim_start().len();
                        line.split_at(i+s)
                    })
                    .unwrap_or((line, ""))
            })($line)
        }};
    }
    match dump {
        false => {
            let mut line = String::new();
            let mut stdin = stdin().lock();
            let mut lnum = 0u64;
            loop {
                line.clear();
                if 0 == stdin.read_line(&mut line)? { break Ok(()) }
                lnum += 1;

                let (prefix, line) = skip_ch_line!(&line);
                let result = if !hard {
                    number(line)
                } else {
                    hard_number(line)
                };
                let (n, rem_str) = result
                    .map(|(n, s)| (Some(n), s))
                    .or_else(|e| {
                        writeln!(stderr(), "`{}` {lnum}:{} expected {}",
                            line.trim_end(),
                            e.location.column+skip_ch,
                            e.expected,
                        )?;
                        io::Result::Ok((None, line))
                    })?;
                let mut stdout = stdout().lock();
                if rem {
                    write!(stdout, "{prefix}")?;
                }
                if let Some(n) = n {
                    write!(stdout, "{n}")?;
                }
                if rem {
                    write!(stdout, "{rem_str}")?;
                } else {
                    write!(stdout, "{}", get_eol(rem_str))?;
                }
            }
        },
        true => {
            let mut line = String::new();
            let mut stdin = stdin().lock();
            let mut lnum = 0u64;
            loop {
                line.clear();
                if 0 == stdin.read_line(&mut line)? { break Ok(()) }
                lnum += 1;

                let (prefix, line) = skip_ch_line!(&line);
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
                            "`{part}` ({}) {lnum}: {e}",
                            rem_str.trim_end(),
                        )?;
                        io::Result::Ok(None)
                    })?;

                let mut stdout = stdout().lock();
                if rem {
                    write!(stdout, "{prefix}")?;
                }
                if let Some(num) = num {
                    cfg.num_fmt()(&mut stdout, num)?;
                } else {
                    write!(stdout, "{part}")?;
                }
                if rem {
                    write!(stdout, "{rem_str}")?;
                } else {
                    write!(stdout, "{}", get_eol(rem_str))?;
                }
            }
        },
    }
}
