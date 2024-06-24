use std::{fmt, cell::Cell};

peg::parser!(pub grammar parser() for str {
    pub rule one_num(d: u32) -> u32
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
    rule power_num() -> u32
        = "亿" { 100000000 }
        / "万" { 10000 }
    rule k_number() -> u32
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
    #[no_eof]
    pub rule number() -> u32
        = (s:$(['0'..='9']+) {? s.parse().map_err(|_| "valid-number") })
        / k:k_number() rest:(p:power_num() k:k_number()? { (p, k.unwrap_or_default()) })*
        {
            let (num, high_pow) = rest.into_iter()
                .rfold((0, 1), |(rnum, pow), (lpow, num)| {
                    (num*pow+rnum, lpow)
                });
            num + k*high_pow
        }
});

pub fn to_zh_num(num: u32) -> String {
    fn one(n: u32) -> char {
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
        num: u32,
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
            let digit = num / 10u32.pow(pow_d) % 10;
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
        num: u32,
        sp: &mut Option<bool>,
        pow_i: u32,
        pow_ch: char,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let [a, b] = [num / pow_i, num % pow_i];
        write!(f, "{}", FmtNum(a, Cell::new(sp.into())))?;
        write!(f, "{pow_ch}")?;
        write!(f, "{}", FmtNum(b, Cell::new(sp.into())))?;
        Ok(())
    }
    struct FmtNum<'a>(u32, Cell<Option<&'a mut Option<bool>>>);
    impl fmt::Display for FmtNum<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let num = self.0;
            let sp = self.1.take().unwrap();
            match num {
                0..=9999 => unit(num, sp, f),
                1_0000..=9999_9999 => {
                    concat_unit(num, sp, 1_0000, '万', f)
                },
                1_0000_0000..=u32::MAX => {
                    concat_unit(num, sp, 1_0000_0000, '亿', f)
                },
            }
        }
    }
    if num == 0 {
        return one(0).into();
    }
    FmtNum(num, Cell::new(Some(&mut None))).to_string()
}

#[cfg(test)]
mod tests {
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
        ];
        for (src, num) in datas {
            assert_eq!(parser::number(src), Ok(num), "{src} -> {num}");
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
        ];
        for (src, num) in datas {
            assert_eq!(to_zh_num(src), num, "{src} -> {num}");
        }
    }
}
