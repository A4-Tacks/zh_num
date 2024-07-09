use std::{
    env::args,
    io::{self, stderr, stdin, stdout, Write},
    process::exit,
};

use zh_num::{parser::number, ZhNum, ZhNumUpper};

const NAME: &str = env!("CARGO_BIN_NAME");

fn main() -> io::Result<()> {
    let mut rem = false;
    let mut skip_ch = 0;
    let mut num_fmt: fn(&mut io::StdoutLock, _) -> io::Result<()>
        = |f: &mut io::StdoutLock, n| {
            write!(f, "{}", ZhNum(n))
        };
    let args = args()
        .skip(1)
        .filter(|arg| (arg == "-r").then(|| rem = true).is_none())
        .filter(|arg| arg.starts_with("-s").then(|| {
            skip_ch = arg[2..].parse().expect("Invalid -s")
        }).is_none())
        .map(|arg| (arg == "-D").then(|| {
            num_fmt = |f, n| { write!(f, "{}", ZhNumUpper(n)) };
            "-d".to_owned()
        }).unwrap_or(arg))
        .collect::<Vec<_>>();
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
                    .map(|(i, _)| line.split_at(i))
                    .unwrap_or((line, ""))
            })($line)
        }};
    }
    let args = args.iter().map(|x| &**x).collect::<Vec<_>>();
    match args[..] {
        [] => {
            stdin().lines().try_for_each(|line| {
                (|line| {
                    let (prefix, line) = skip_ch_line!(line);
                    let (n, s) = number(line)
                        .map(|(n, s)| (Some(n), s))
                        .or_else(|e| {
                            writeln!(stderr(), "`{line}`:{} expected {}",
                                e.location.column,
                                e.expected
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
                        write!(stdout, "{s}")?;
                    }
                    writeln!(stdout)?;
                    Ok(())
                })(line?.trim())
            })
        },
        ["-h" | "--help", ..] | [.., "-h" | "--help"] => {
            eprintln!("USAGE: {NAME} [-r | -s]... [-d | -D] [-h | --help | -v | --version]");
            eprintln!("将ASCII数字和中文数字相互转换");
            eprintln!("OPTIONS:");
            eprintln!("    -d               反向转换, 也就是将ASCII数字转换成中文数字");
            eprintln!("    -D               类似 -d, 但是中文数字是大写");
            eprintln!("    -r               转换时保留结果之后的文本");
            eprintln!("    -s<num>          识别时跳过一部分字符, 如果给定了-r则会留在结果中");
            eprintln!("    -v, --version    显示版本信息");
            eprintln!("    -h, --help       显示帮助");
            Ok(())
        },
        ["-v" | "--version", ..] | [.., "-v" | "--version"] => {
            eprintln!("{NAME}@v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        },
        ["-d"] => {
            stdin().lines().try_for_each(|line| {
                let line = line?;
                let (prefix, line) = skip_ch_line!(line.trim());
                let rem_idx = line
                    .find(|ch| !char::is_ascii_digit(&ch))
                    .unwrap_or(line.len());
                let (part, rem_str) = line.split_at(rem_idx);
                let num = part
                    .parse()
                    .or_else(|e| {
                        writeln!(stderr(), "`{line}` ({part}) {e}")?;
                        io::Result::Ok(0)
                    })?;

                let mut stdout = stdout().lock();
                if rem {
                    write!(stdout, "{prefix}")?;
                }
                num_fmt(&mut stdout, num)?;
                if rem {
                    write!(stdout, "{rem_str}")?;
                }
                write!(stdout, "\n")?;
                Ok(())
            })
        },
        _ => {
            eprintln!("Invalid args, run `{NAME} -h` show help");
            exit(2)
        },
    }
}
