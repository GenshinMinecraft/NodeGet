use vergen_gix::{BuildBuilder, CargoBuilder, Emitter, GixBuilder, RustcBuilder};

fn main() {
    let build = BuildBuilder::default().build_timestamp(true).build().unwrap();
    let cargo = CargoBuilder::default().target_triple(true).build().unwrap();
    let gix = GixBuilder::default()
        .branch(true)
        .sha(true)
        .commit_message(true)
        .commit_timestamp(true)
        .build()
        .unwrap();
    let rustc = RustcBuilder::default()
        .channel(true)
        .semver(true)
        .commit_date(true)
        .commit_hash(true)
        .llvm_version(true)
        .build()
        .unwrap();

    Emitter::default()
        .add_instructions(&build)
        .unwrap()
        .add_instructions(&cargo)
        .unwrap()
        .add_instructions(&gix)
        .unwrap()
        .add_instructions(&rustc)
        .unwrap()
        .emit()
        .unwrap();
}
