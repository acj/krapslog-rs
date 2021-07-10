# krapslog

[![Actions Status](https://github.com/acj/krapslog-rs/workflows/CI/badge.svg)](https://github.com/acj/krapslog-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/krapslog.svg)](https://crates.io/crates/krapslog)

Visualize a log file with [sparklines](https://en.wikipedia.org/wiki/Sparkline)

When troubleshooting a problem with a production service, I often need to get the general shape of a log file. Are there any spikes? Was the load higher during the incident than it was beforehand? Does anything else stand out? Without tooling to help you, a large log file is little more than a blob of data. This tool is designed to quickly surface key features of the log — and then get out of your way.

## Installing

### Homebrew

```
brew install acj/taps/krapslog
```

### From source

```
cargo install krapslog
```

## Usage

```
$ krapslog -h
[...]
Visualize log files using sparklines

USAGE:
    krapslog [FLAGS] [OPTIONS] [FILE]

FLAGS:
    -p, --progress    Display progress while working. Requires a file.
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -F, --format <FORMAT>      Timestamp format to match [default: %d/%b/%Y:%H:%M:%S%.f]
    -m, --markers <MARKERS>    Number of time markers to display [default: 0]

ARGS:
    <FILE>    Log file to analyze
```

## Examples

Get the basic shape:

```
$ krapslog /var/log/haproxy.log 
▂▂▂▂▂▁▂▁▁▁▁▂▁▁▁▁▂▂▂▁▁▁▁▁▁▁▁▁▂▂▂▂▂▂▂▂▂▃▂▂▂▃▂▂▂▂▃▃▃▃▃▄▅▅▅▄▅▃▄▃▄▄▅▅▆▇▆▆▆▆▆▆▆▆▇▇▇▇██
```

Add points in time:

```
$ krapslog --markers 10 /var/log/haproxy.log
                                                             Sat Nov 23 14:15:56
                                                    Sat Nov 23 13:22:29        |
                                           Sat Nov 23 12:29:01        |        |
                                  Sat Nov 23 11:35:33        |        |        |
                          Sat Nov 23 10:48:02       |        |        |        |
                                            |       |        |        |        |
▂▂▂▂▂▁▂▁▁▁▁▂▁▁▁▁▂▂▂▁▁▁▁▁▁▁▁▁▂▂▂▂▂▂▂▂▂▃▂▂▂▃▂▂▂▂▃▃▃▃▃▄▅▅▅▄▅▃▄▃▄▄▅▅▆▇▆▆▆▆▆▆▆▆▇▇▇▇██
|        |        |       |        |
|        |        |       |        Sat Nov 23 09:54:34
|        |        |       Sat Nov 23 09:01:07
|        |        Sat Nov 23 08:13:36
|        Sat Nov 23 07:20:08
Sat Nov 23 06:26:40
```

Integrate with other tools:

```
$ zcat /var/log/haproxy.log.1.gz | grep -v "unimportant.html" | krapslog
▂▁▂▁▂▁▂▂▂▁▃▁▁▁▁▁▁▁▁▁▁▁▂▂▁▁▂▃▂▂▃▁▂▁▂▂▂▂▁▂▁▂▄▂▂▂▂▂▂▂▃▂▂▂▂▄▃▃▄▃▃▃▃▄▄▄▄▄▃▄▄▅▄▃▄▄▅▅▅▅
```

## Custom date formats

By default, krapslog assumes that log timestamps are in the [Common Log Format (CLF)](https://httpd.apache.org/docs/1.3/logs.html#common), which looks like this: "02/Jan/2006:15:04:05.000" (timezone offset is ignored). However, you can use the `format` parameter to find timestamps in other formats. The parameter value must use a format that's recognized by [strftime](https://docs.rs/chrono/0.4.13/chrono/format/strftime/index.html).

For example, if your log contains dates that look like  "Jan 1, 2020 15:04:05", you can run krapslog as follows:

```
krapslog --format "%b %d, %Y %H:%M:%S" ...
```

### Currently supported specifiers

| Specifier | Meaning |
| --------- | ------- |
| %Y        | The full proleptic Gregorian year, zero-padded to 4 digits. |
| %C        | The proleptic Gregorian year divided by 100, zero-padded to 2 digits. |
| %y        | The proleptic Gregorian year modulo 100, zero-padded to 2 digits. |
| %m        | Month number (01--12), zero-padded to 2 digits. |
| %b        | Abbreviated month name. Always 3 letters. |
| %B        | Full month name. Also accepts corresponding abbreviation in parsing. |
| %h        | Same as %b. |
| %d        | Day number (01--31), zero-padded to 2 digits. |
| %H        | Hour number (00--23), zero-padded to 2 digits. |
| %M        | Minute number (00--59), zero-padded to 2 digits. |
| %S        | Second number (00--60), zero-padded to 2 digits. |
| %.f       | Similar to .%f but left-aligned. These all consume the leading dot. |
| %s        | UNIX timestamp. Seconds since 1970-01-01 00:00 UTC. |

## Contributing

Please be kind. We're all trying to do our best.

If you find a bug, please open an issue. (Or, better, submit a pull request that fixes it!)

If you'd like see a new feature or would like to add one yourself, please open an issue so that we can discuss it.
