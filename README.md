# AWSCREDX
[![Build Status](https://travis-ci.org/sam701/awscredx.svg?branch=master)](https://travis-ci.org/sam701/awscredx)

Assume AWS roles under the motto "don't make me think".
`awscredx` has a goal to make the role assumption on the command line simple and intuitive.

## How to use it
Download the [binary](https://github.com/sam701/awscredx/releases/latest).
Add `awscredx` into your `PATH` and call `awscredx init`.
It will print what it has done.

The `awscredx init` sets up a shell script with the function `assume`.
In a new shell you can call `assume <profile name>` to assume the role from `<profile name>`. 

## Features

### Explains what has been done
![init](./doc/init.png)

### Configurable role profiles
The [configuration file](./src/init/config-template.toml) is well documented.
```toml
[profiles]
dev = "arn:aws:iam::123456589012:role/Admin"
prod = "arn:aws:iam::123456589013:role/TestRole"
```

### Shows assumed profile in shell prompt
![prompt](./doc/prompt.png)

### Yubikey integration
The MFA is read from your Yubikey so you do not need to type it.\
![prompt](./doc/yubikey.png)

### Checks for new versions
![version-check](./doc/version-check.png)

### Written with love in Rust
https://www.rust-lang.org/

## License
MIT