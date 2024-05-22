extern crate cargo_lock;
extern crate napi_build;

use cargo_lock::Lockfile;
use std::collections::HashMap;

fn main() {
  version();
  napi_build::setup();
}

fn version() {
  println!("cargo:rerun-if-changed=Cargo.lock");

  let lock: Lockfile = include_str!("Cargo.lock").parse().unwrap();
  let packages: HashMap<_, _> = lock.packages.iter().map(|p| (p.name.as_str(), p)).collect();

  println!(
    "cargo:rustc-env=JPREPROCESS_VERSION={}",
    packages.get("jpreprocess").unwrap().version
  );
  println!(
    "cargo:rustc-env=JBONSAI_VERSION={}",
    packages.get("jbonsai").unwrap().version
  );
}
