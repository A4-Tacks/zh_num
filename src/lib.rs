use std::{fmt, cell::Cell};

pub type Number = u64;

peg::parser!(pub grammar parser() for str {
    pub rule one_num(d: Number) -> Number
        = "零" n:one_num(d)? { n.unwrap_or(d) }
        / "一" { 1 }
        / "二" { 2 }
        / "两" { 2 }
        / "三" { 3 }
        / "四" { 4 }
        / "五" { 5 }
        / "六" { 6 }
        / "七" { 7 }
        / "八" { 8 }
        / "九" { 9 }
    rule power_num() -> Number
        = "亿" { 100000000 }
        / "万" { 10000 }
    rule k_number() -> Number
        = a:(n:one_num(0) "千" { 1000 * n })?
          b:(n:one_num(0) "百" { 100 * n })?
          c:(n:one_num(1)?"十" { 10 * n.unwrap_or(1) })?
          d:(n:one_num(0)      { n })?
        {?
            [a, b, c, d].into_iter()
                .flatten()
                .reduce(|a, b| a + b)
                .ok_or("num-unit")
        }
    rule wan_number() -> Number
        = w:k_number() n:("万" n:k_number()? { n.unwrap_or_default() })?
        {
            n.map(|n| w * 10000 + n)
                .unwrap_or(w)
        }
    rule yi_number() -> Number
        = w:wan_number() rest:("亿" x:wan_number()? { x.unwrap_or_default() })*
        {
            rest.into_iter().fold(w, |high, n| {
                high * 1_0000_0000 + n
            })
        }
    rule raw_number() -> Number
        = (s:$(['0'..='9']+) {? s.parse().map_err(|_| "valid-number") })
        / yi_number()
    /// Parse zh nums, return parsed number and rest text
    ///
    /// # Examples
    /// ```
    /// # use zh_num::parser::number;
    /// assert_eq!(number("一万零十三章"), Ok((10013, "章")));
    /// ```
    pub rule number() -> (Number, &'input str)
        = n:raw_number() s:$([_]*)
        { (n, s) }
});

/// [`to_zh_num`] write to [`Write`] impl
///
/// [`Write`]: fmt::Write
pub fn fmt_zh_num(num: Number, mut f: impl fmt::Write) -> fmt::Result {
    fn one(n: Number) -> char {
        match n {
            0 => '零',
            1 => '一',
            2 => '二',
            3 => '三',
            4 => '四',
            5 => '五',
            6 => '六',
            7 => '七',
            8 => '八',
            9 => '九',
            _ => panic!("{n}"),
        }
    }
    fn unit(
        num: Number,
        sp: &mut Option<bool>,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        assert!(num < 10000, "{num}");

        const P: [Option<char>; 4] = [
            None,
            Some('十'),
            Some('百'),
            Some('千'),
        ];

        for (pow_d, p) in (0..4).zip(P).rev() {
            let digit = num / Number::pow(10, pow_d) % 10;
            let digit_ch = one(digit);
            if digit == 0 {
                if let Some(x) = sp { *x = true }
                continue;
            }
            if let Some(true) = sp { write!(f, "零")? }
            if !(sp.is_none() && digit == 1 && p == Some('十')) {
                write!(f, "{digit_ch}")?;
            }
            if let Some(p) = p {
                write!(f, "{p}")?;
            }
            *sp = Some(false)
        }
        Ok(())
    }
    fn concat_unit(
        num: Number,
        sp: &mut Option<bool>,
        pow_i: Number,
        pow_ch: char,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let [a, b] = [num / pow_i, num % pow_i];
        write!(f, "{}", FmtNum(a, Cell::new(sp.into())))?;
        write!(f, "{pow_ch}")?;
        write!(f, "{}", FmtNum(b, Cell::new(sp.into())))?;
        Ok(())
    }
    struct FmtNum<'a>(Number, Cell<Option<&'a mut Option<bool>>>);
    impl fmt::Display for FmtNum<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let num = self.0;
            let sp = self.1.take().unwrap();
            match num {
                0..=9999 => unit(num, sp, f),
                1_0000..=9999_9999 => {
                    concat_unit(num, sp, 1_0000, '万', f)
                },
                1_0000_0000..=Number::MAX => {
                    concat_unit(num, sp, 1_0000_0000, '亿', f)
                },
            }
        }
    }
    if num == 0 {
        return write!(f, "{}", one(0));
    }
    write!(f, "{}", FmtNum(num, Cell::new(Some(&mut None))))
}

/// Convert number to zh words
///
/// # Examples
/// ```
/// # use zh_num::to_zh_num;
/// assert_eq!(to_zh_num(10086), "一万零八十六");
/// ```
pub fn to_zh_num(num: Number) -> String {
    let mut s = String::new();
    fmt_zh_num(num, &mut s).unwrap();
    s
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn test_parse() {
        let datas = [
            ("零", 0),
            ("一", 1),
            ("十", 10),
            ("二十", 20),
            ("二百二十", 220),
            ("两千零一", 2001),
            ("两千一", 2001),
            ("两千一百", 2100),
            ("零万", 0),
            ("三万", 30000),
            ("十万", 100000),
            ("十一", 11),
            ("二十一", 21),
            ("三百六十八", 368),
            ("一万两千三百四十五", 12345),
            ("一亿两千三百四十五万六千七百八十九", 123456789),
            ("五十五", 55),
            ("五十", 50),
            ("一十", 10),
            ("三十", 30),
            ("六百六十六", 666),
            ("一万一", 10001),
            ("一亿一", 100000001),
            ("一亿零一", 100000001),
            ("十亿零一", 1000000001),
            ("十四亿零一", 1400000001),
            ("一十四亿零一", 1400000001),
            ("二十四亿零一", 2400000001),
            ("九十四亿零一", 9400000001),
            ("一百零四亿零一", 10400000001),
            ("一百四亿零一", 10400000001),
            ("一千零四亿零一", 100400000001),
            ("一千亿零一", 100000000001),
            ("一千亿", 100000000000),
            ("一万亿零一", 1000000000001),
            ("一万一百", 10100),
            ("一万零一百", 10100),
            ("一万一百一", 10101),
            ("一万零一百一", 10101),
            ("一万零一百零一", 10101),
            ("一万零一", 10001),
            ("一千零一", 1001),
            ("一百零一", 101),
            ("一十零一", 11),
            ("十零一", 11),
            ("一万十", 10010),
            ("一万零十", 10010),
            ("一万零一十", 10010),
            ("一万零一十三", 10013),
            ("一万零十三", 10013),
            ("一万两千", 12000),
            ("一万两千否", 12000),
            ("零否", 0),
            ("一亿", 1_0000_0000),
            ("一亿零一", 1_0000_0001),
            ("一亿零一十", 1_0000_0010),
            ("一亿零一百", 1_0000_0100),
            ("一亿零一千", 1_0000_1000),
            ("一亿零一万", 1_0001_0000),
            ("一亿零十万", 1_0010_0000),
            ("一亿零一十万", 1_0010_0000),
            ("一万亿零一十万", 1_0000_0010_0000),
            ("一亿亿", 1_0000_0000_0000_0000),
            ("一亿亿零一", 1_0000_0000_0000_0001),
            ("一亿三亿零一", 1_0000_0003_0000_0001),
            ("一亿零三亿零一", 1_0000_0003_0000_0001),
        ];
        for (src, num) in datas {
            assert_eq!(parser::number(src).map(|x| x.0), Ok(num), "{src} -> {num}");
        }
    }

    #[test]
    fn test_to_zh() {
        let datas = [
            (0, "零"),
            (2, "二"),
            (9, "九"),
            (10, "十"),
            (11, "十一"),
            (20, "二十"),
            (21, "二十一"),
            (100, "一百"),
            (101, "一百零一"),
            (109, "一百零九"),
            (110, "一百一十"),
            (111, "一百一十一"),
            (121, "一百二十一"),
            (120, "一百二十"),
            (220, "二百二十"),
            (999, "九百九十九"),
            (990, "九百九十"),
            (909, "九百零九"),
            (1000, "一千"),
            (1001, "一千零一"),
            (1010, "一千零一十"),
            (1011, "一千零一十一"),
            (10000, "一万"),
            (10011, "一万零一十一"),
            (10021, "一万零二十一"),
            (20021, "二万零二十一"),
            (200021, "二十万零二十一"),
            (210021, "二十一万零二十一"),
            (210210, "二十一万零二百一十"),
            (212100, "二十一万二千一百"),
            (212101, "二十一万二千一百零一"),
            (883868, "八十八万三千八百六十八"),
            (1_0000_0000, "一亿"),
            (1_0000_0001, "一亿零一"),
            (1_1000_0000, "一亿一千万"),
            (1_0100_0000, "一亿零一百万"),
            (1_0010_0000, "一亿零一十万"),
            (1_0001_0000, "一亿零一万"),
            (10_0001_0000, "十亿零一万"),
            (100_0001_0000, "一百亿零一万"),
            (1000_0001_0000, "一千亿零一万"),
            (1_0000_0001_0000, "一万亿零一万"),
            (10_0000_0001_0000, "十万亿零一万"),
            (100_0000_0001_0000, "一百万亿零一万"),
            (1000_0000_0001_0000, "一千万亿零一万"),
            (1_0000_0000_0001_0000, "一亿亿零一万"),
            (10_0000_0000_0001_0000, "十亿亿零一万"),
            (14_0000_0000_0001_0000, "十四亿亿零一万"),
            (10_1000_0000_0001_0000, "十亿零一千万亿零一万"),
            (100_0000_0000_0001_0000, "一百亿亿零一万"),
            (200_0000_0000_0001_0000, "二百亿亿零一万"),
            (1000_0000_0000_0001_0000, "一千亿亿零一万"),
            (1300_0000_0000_0001_0000, "一千三百亿亿零一万"),
            (1030_0000_0000_0001_0000, "一千零三十亿亿零一万"),
            (1003_0000_0000_0001_0000, "一千零三亿亿零一万"),
            (1003_3000_0000_0001_0000, "一千零三亿三千万亿零一万"),
            (1003_0300_0000_0001_0000, "一千零三亿零三百万亿零一万"),
            (1_0000_0000_0000, "一万亿"),
            (1_0001_0000_0000, "一万零一亿"),
            (1_0000_0000_0000_0000, "一亿亿"),
        ];
        for (src, num) in datas {
            assert_eq!(to_zh_num(src), num, "{src} -> {num}");
        }
    }

    #[test]
    #[ignore = "long-time-test"]
    fn test_num_range() {
        let thread_count = thread::available_parallelism()
            .unwrap_or(1.try_into().unwrap());
        let groups = Number::MAX as usize / thread_count;
        let handles = (0..thread_count.get())
            .map(|g| {
                thread::spawn(move || {
                    let mut s = String::new();
                    (g*groups..(g+1).saturating_mul(groups))
                        .step_by(141)
                        .for_each(|n|
                    {
                        let n = n as Number;
                        s.clear();
                        fmt_zh_num(n, &mut s).unwrap();
                        let num
                            = parser::number(&s);
                        assert_eq!(num.map(|m| m.0), Ok(n));
                    });
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
