<a name=""></a>
##  v0.5.0 (2016-04-20)


#### Features

*   add `--only` option which restricts entries to either dirs or files ([9056b940](https://gitlab.com/smaximov/ded/commit/9056b9407c09a0bf32a9f295d6445d4167e975a2), closes [#3](https://gitlab.com/smaximov/ded/issues/3))
*   abort if a directory listing appears to be empty ([f0030f85](https://gitlab.com/smaximov/ded/commit/f0030f8558be4b5142c629b24667b6641f8fc9f3), closes [#1](https://gitlab.com/smaximov/ded/issues/1))
*   add option to control ded's temp directory ([428df506](https://gitlab.com/smaximov/ded/commit/428df506d79fb9c52dbedec4e9a208687c0256b5), closes [#4](https://gitlab.com/smaximov/ded/issues/4))
*   add option `--match` to list only entries which match any of the provided glob patterns ([7d459344](https://gitlab.com/smaximov/ded/commit/7d45934455e4440c0d25a06d129b94fd81fbf094))



<a name=""></a>
##  v0.4.0 (2016-04-17)


#### Features

*   add `--dry-run` flag ([711b148c](https://gitlab.com/smaximov/ded/commit/711b148c124d88abeb06f66e5a0e4082575a6884))
*   set the default answer to prompts using flags `--yes` and `--no` ([8df22abf](https://gitlab.com/smaximov/ded/commit/8df22abfc285209dda0c632f52f2b568eb7b4f6b))
*   add option to enable verbose output ([ac5e6027](https://gitlab.com/smaximov/ded/commit/ac5e6027d8128917ff045bd95ec834bf91c77ed9))
*   ask when trying to overwrite existing entries ([29937480](https://gitlab.com/smaximov/ded/commit/29937480a63940d6a0eb90980188409b1dd5f589))



<a name=""></a>
##  v0.3.0 (2016-04-15)


#### Features

*   append trailing slash (fs separator) to directories when editing entries ([200ad712](https://gitlab.com/smaximov/ded/commit/200ad712d02ac48ed2b0c3d98efa6fecfd932bb5))
*   implement the removal of files and directories ([cf70f72d](https://gitlab.com/smaximov/ded/commit/cf70f72d888c5e168f95c5c10e07428be7964df5))



<a name=""></a>
##  v0.2.0 (2016-04-12)


#### Bug Fixes

*   exit with failure when edit fails ([cf163cf1](https://gitlab.com/smaximov/ded/commit/cf163cf176d4f4c7e2bd5d193507a050af6211de))

#### Features

*   add option to display hidden files and dirs (disabled by default) ([289df4bd](https://gitlab.com/smaximov/ded/commit/289df4bdd49993142a4895ff6a2a111f7642737d))
*   add option to set editor ([627093e1](https://gitlab.com/smaximov/ded/commit/627093e1eabb5b693d50cc117a39072de5f96a8b))
*   add option to set working directory ([2c67894b](https://gitlab.com/smaximov/ded/commit/2c67894b26dbce6bef632cd7f8458c345d3c2db7))



<a name=""></a>
##  v0.1.0 (2016-04-12)


#### Features

*   rename of directory entries with patterns support ([9b896d56](https://gitlab.com/smaximov/ded/commit/9b896d56744d2307519d85e8385a5d6be4e8fba9))
