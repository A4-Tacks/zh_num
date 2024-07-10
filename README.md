转换ASCII数字和中文数字的实用程序

# Examples
```shell
$ zh_num -h
Usage: zh_num [OPTIONS]
将ASCII数字和中文数字相互转换

Options:
  -d                反向转换, 也就是将ASCII数字转换成中文数字
  -D                类似 -d, 但是中文数字是大写
  -r                转换时保留结果之外的文本
  -s <SKIP_CH>      识别时跳过一部分字符, 如果给定了-r则会留在结果中 [default: 0]
  -v, --version     Print version
  -h, --help        Print help

zh_num@0.3.5
A4-Tacks <wdsjxhno1001@163.com>
$ zh_num
1234
1234
一千二百三十四
1234
六万
60000
十三
13
肆佰贰拾
420
$ zh_num -d
1234
一千二百三十四
286639
二十八万六千六百三十九
10086
一万零八十六
$ zh_num -D
2333
贰仟叁佰叁拾叁
$ zh_num -ds1
第2章
二
$ zh_num -drs1
第2章
第二章
$ zh_num -rs1
第四章
第4章
```

Install
===============================================================================
Install executable file:

```shell
cargo install zh_num --features bin
```

Add as a dependencies:

```shell
cargo add zh_num
```
