use std::{
    env::args,
    io::{self, stderr, stdin, stdout, Write},
    process::exit,
};

use zh_num::{parser::number, to_zh_num};

const NAME: &'static str = env!("CARGO_BIN_NAME");

fn main() -> io::Result<()> {
    let args = args().skip(1).collect::<Vec<_>>();
    let args = args.iter().map(|x| &**x).collect::<Vec<_>>();
    match args[..] {
        [] => {
            stdin().lines().try_for_each(|line| {
                (|line| {
                    writeln!(stdout(), "{}", number(line)
                        .or_else(|e| {
                            writeln!(stderr(), "`{line}`:{} expected {}",
                                e.location.column,
                                e.expected
                            )?;
                            io::Result::Ok(0)
                        })?
                    )?;
                    Ok(())
                })(line?.trim())
            })
        },
        ["-h" | "--help", ..] | [.., "-h" | "--help"] => {
            eprintln!("USAGE: {NAME} [-d] [-h | --help]");
            eprintln!("将ASCII数字和中文数字相互转换");
            eprintln!("OPTIONS:");
            eprintln!("    -d           反向转换, 也就是将ASCII数字转换成中文数字");
            eprintln!("    -h, --help   显示帮助");
            Ok(())
        },
        ["-d"] => {
            stdin().lines().try_for_each(|line| {
                let line = line?;
                let line = line.trim();
                let part = line
                    .split_once(|ch: char| !ch.is_ascii_digit())
                    .map(|(s, _)| s)
                    .unwrap_or(line);
                let num = part
                    .parse()
                    .or_else(|e| {
                        writeln!(stderr(), "`{line}` ({part}) {e}")?;
                        io::Result::Ok(0)
                    })?;
                writeln!(stdout(), "{}", to_zh_num(num))
            })
        },
        _ => {
            eprintln!("Invalid args, run `{NAME} -h` show help");
            exit(2)
        },
    }
}
