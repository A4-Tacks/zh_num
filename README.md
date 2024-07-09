转换ASCII数字和中文数字的实用程序

# Examples
```shell
$ zh_num -h
USAGE: zh_num [-r | -s]... [-d] [-h | --help]
将ASCII数字和中文数字相互转换
OPTIONS:
    -d               反向转换, 也就是将ASCII数字转换成中文数字
    -r               转换时保留结果之后的文本
    -s<num>          识别时跳过一部分字符, 如果给定了-r则会留在结果中
    -v, --version    显示版本信息
    -h, --help       显示帮助
$ zh_num 
1234
1234
一千二百三十四
1234
六万
60000
十三
13
$ zh_num -d
1234
一千二百三十四
286639
二十八万六千六百三十九
10086
一万零八十六
$ zh_num -d -s1
第2章
二
$ zh_num -d -s1 -r
第2章
第二章
$ zh_num -s1 -r
第四章
第4章
$ 
```
