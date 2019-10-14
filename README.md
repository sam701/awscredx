# AWSCREDX
[![Build Status](https://travis-ci.org/sam701/awscredx.svg?branch=master)](https://travis-ci.org/sam701/awscredx)

Assume AWS roles under the motto "don't make me think".
`awscredx` has a goal to make the role assumption on the command line simple and intuitive.

## How to use it
Download the [binary](https://github.com/sam701/awscredx/releases/latest).
Add `awscredx` into your `PATH` and call `awscredx init`.
It will print what it has done.

TODO: add a screenshot

The `awscredx init` sets up a shell script with the function `assume`.
In a new shell you can call `assume <profile name>` to assume the role from `<profile name>`. 