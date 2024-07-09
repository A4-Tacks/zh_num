use std::{
    convert::Infallible,
    env::args,
    fmt,
    io::{self, stderr, stdin, stdout, Write},
    process::exit,
};

use zh_num::{fmt_zh_num, parser::number};

struct IOFmtWrapper<W> {
    write: W,
    err: Option<io::Result<Infallible>>,
}
impl<W: io::Write> IOFmtWrapper<W> {
    fn new(write: W) -> Self {
        Self { write, err: None }
    }

    fn err(self) -> io::Result<()> {
        match self.err {
            Some(Ok(_)) => unreachable!(),
            Some(Err(e)) => Err(e),
            None => Ok(()),
        }
    }

    fn io_write_fmt(&mut self, args: fmt::Arguments<'_>) {
        if self.err.is_some() { return; }
        if let Err(e) = io::Write::write_fmt(self.write.by_ref(), args) {
            self.err.get_or_insert(Err(e));
        }
    }

    fn io_write_str(&mut self, s: &str) {
        self.io_write_fmt(format_args!("{s}"))
    }
}
impl<W: io::Write> fmt::Write for IOFmtWrapper<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.io_write_fmt(format_args!("{s}"));
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.io_write_fmt(args);
        Ok(())
    }
}

const NAME: &str = env!("CARGO_BIN_NAME");

fn main() -> io::Result<()> {
    let mut rem = false;
    let mut skip_ch = 0;
    let args = args()
        .skip(1)
        .filter(|arg| (arg == "-r").then(|| rem = true).is_none())
        .filter(|arg| arg.starts_with("-s").then(|| {
            skip_ch = arg[2..].parse().expect("Invalid -s")
        }).is_none())
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
            eprintln!("USAGE: {NAME} [-r | -s]... [-d] [-h | --help]");
            eprintln!("将ASCII数字和中文数字相互转换");
            eprintln!("OPTIONS:");
            eprintln!("    -d           反向转换, 也就是将ASCII数字转换成中文数字");
            eprintln!("    -r           转换时保留结果之后的文本");
            eprintln!("    -s<num>      识别时跳过一部分字符, 如果给定了-r则会留在结果中");
            eprintln!("    -h, --help   显示帮助");
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

                let mut stdout = IOFmtWrapper::new(stdout().lock());
                if rem {
                    stdout.io_write_str(prefix);
                }
                fmt_zh_num(num, &mut stdout).unwrap();
                if rem {
                    stdout.io_write_str(rem_str);
                }
                stdout.io_write_str("\n");
                stdout.err()
            })
        },
        _ => {
            eprintln!("Invalid args, run `{NAME} -h` show help");
            exit(2)
        },
    }
}
