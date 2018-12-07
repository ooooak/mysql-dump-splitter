sql-split
------------
sql-split is a command line tool to split SQL Dump file into small files.


Usage 
----------

```bash
$ sql-split.exe file.sql --output=20mb
```

> Windows Binary is under `bin` dir.

#### How to build
> cargo build --release

TODO:
1. ~~`parse cli params `output_size`~~
2. ~~write tests~~
3. ~~fix unsafe code. remove die~~
4. better error reporting
5. add multi threading
