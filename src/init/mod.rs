use std::fs::{self, File};
use std::io::prelude::*;
use std::process::Command;
use std::{env, process};

use tera::{Context, Tera};

use crate::util;
use crate::version::VERSION;

mod context;
mod init;
pub mod setup;
mod styles;

pub use init::*;
