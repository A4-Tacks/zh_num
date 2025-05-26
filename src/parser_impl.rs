use crate::Number;

peg::parser!(pub grammar parser() for str {
    use vec;

    rule c10() = quiet!{['十' | '拾' | '⑩' | '⑽' | '㈩']} / expected!("十")
    rule c20() = quiet!{['百' | '佰' | '陌']} / expected!("百")
    rule c30() = quiet!{['千' | '仟' | '阡']} / expected!("千")
    rule one_num_inner(d: Number) -> Number
        = ['零' | '０' | '〇'] n:one_num(d)?   { n.unwrap_or(d) }
        / ['一' | '１' | '⑴' | '㈠' | '①' | '壹' | '弌' | '幺'] { 1 }
        / ['二' | '２' | '⑵' | '㈡' | '②' | '贰' | '弍' | '两'] { 2 }
        / ['三' | '３' | '⑶' | '㈢' | '③' | '叁' | '弎']        { 3 }
        / ['四' | '４' | '⑷' | '㈣' | '④' | '肆']               { 4 }
        / ['五' | '５' | '⑸' | '㈤' | '⑤' | '伍']               { 5 }
        / ['六' | '６' | '⑹' | '㈥' | '⑥' | '陆']               { 6 }
        / ['七' | '７' | '⑺' | '㈦' | '⑦' | '柒']               { 7 }
        / ['八' | '８' | '⑻' | '㈧' | '⑧' | '捌']               { 8 }
        / ['九' | '９' | '⑼' | '㈨' | '⑨' | '玖']               { 9 }
    pub rule one_num(d: Number) -> Number
        = quiet!{one_num_inner(d)}
        / expected!("num-unit")
    rule power_num() -> Number
        = "亿" { 1_0000_0000 }
        / "万" { 1_0000 }
    rule k_number() -> Number
        = a:(n:one_num(0)  c30()    { 1000 * n })?
          b:(n:one_num(0)  c20()    { 100 * n })?
          c:(n:one_num(1)? c10()    { 10 * n.unwrap_or(1) })?
          d:(n:one_num(0)           { n })?
        {?
            [a, b, c, d].into_iter()
                .flatten()
                .reduce(|a, b| a + b)
                .ok_or("num-unit")
        }
    rule wan_number() -> Number
        = w:k_number() n:("万" n:k_number()? { n.unwrap_or_default() })?
        {
            n.map_or(w, |n| w * 10000 + n)
        }
    rule yi_number() -> Number
        = w:wan_number() rest:("亿" x:wan_number()? { x.unwrap_or_default() })*
        {
            rest.into_iter().fold(w, |high, n| {
                high * 1_0000_0000 + n
            })
        }
    rule ascii_digits() = quiet!{['0'..='9']+} / expected!("ascii-digit")

    /// Parse zh nums and ascii-digits, return parsed number
    ///
    /// # Examples
    /// ```
    /// # use zh_num::parser::raw_number;
    /// assert_eq!(raw_number("一万零十三"), Ok(10013));
    /// ```
    pub rule raw_number() -> Number
        = (s:$(ascii_digits()) {? s.parse().map_err(|_| "valid-number") })
        / yi_number()

    /// Parse zh nums and ascii-digits, return parsed number and rest text
    ///
    /// # Examples
    /// ```
    /// # use zh_num::parser::number;
    /// assert_eq!(number("一万零十三章"), Ok((10013, "章")));
    /// ```
    pub rule number() -> (Number, &'input str)
        = n:raw_number() s:$([_]*)
        { (n, s) }

    /// Parse hard zh nums, return parsed number
    ///
    /// # Examples
    /// ```
    /// # use zh_num::parser::raw_hard_number;
    /// assert_eq!(raw_hard_number("一零零八六"), Ok(10086));
    /// assert_eq!(raw_hard_number("一零零十三"), Ok(10013));
    /// assert_eq!(raw_hard_number("零零零"), Ok(0));
    /// assert_eq!(raw_hard_number("百零零"), Ok(100));
    /// ```
    pub rule raw_hard_number() -> Number
        = nums:(
            "零" { 0 }
            / (c10() / c20() / c30() / ['万' | '亿']) { 1 }
            / one_num(0))+
        {
            nums.into_iter().fold(0, |acc, num| acc * 10 + num)
        }

    /// Parse hard zh nums, return parsed number and rest text
    ///
    /// # Examples
    /// ```
    /// # use zh_num::parser::hard_number;
    /// assert_eq!(hard_number("一零零八六章"), Ok((10086, "章")));
    /// assert_eq!(hard_number("一零零十三章"), Ok((10013, "章")));
    /// assert_eq!(hard_number("零零零章"), Ok((0, "章")));
    /// assert_eq!(hard_number("百零零章"), Ok((100, "章")));
    /// ```
    pub rule hard_number() -> (Number, &'input str)
        = num:raw_hard_number() s:$([_]*) { (num, s) }
});
