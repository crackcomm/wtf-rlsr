use std::cell::RefCell;

use cargo::{
    core::Workspace as CargoWorkspace,
    ops::{self, CompileFilter, CompileOptions, Packages, TestOptions},
    util::Config as CargoConfig,
};

use crate::{util::Logger, ws::Package};

/// Runs test for all packages in the workspace.
pub fn run_tests(
    package: &Package<'_>,
    workspace: &CargoWorkspace<'_>,
) -> Result<bool, failure::Error> {
    workspace.status("Testing", "Starting");
    let test_opts = test_opts(workspace.config());
    let mut success = true;
    match ops::run_tests(&workspace, &test_opts, &[]) {
        Ok(_) => {
            workspace.status("Testing", "Finished");
        }
        Err(err) => {
            println!(
                "Package {} test returned with error: {}.",
                package.name(),
                err
            );
            success = false;
        }
    };
    std::env::set_var("CARGO_TARGET_DIR", "target/cxmr-rlsr");
    Ok(success)
}

fn test_opts(config: &CargoConfig) -> TestOptions {
    TestOptions {
        compile_opts: CompileOptions {
            config: config,
            build_config: cargo::core::compiler::BuildConfig {
                requested_kind: cargo::core::compiler::CompileKind::Host,
                jobs: 8,
                profile_kind: cargo::core::compiler::ProfileKind::Dev,
                mode: cargo::core::compiler::CompileMode::Test,
                message_format: cargo::core::compiler::MessageFormat::Human,
                force_rebuild: false,
                build_plan: false,
                primary_unit_rustc: None,
                rustfix_diagnostic_server: RefCell::new(None),
            },
            features: vec![],
            all_features: true,
            no_default_features: false,
            spec: Packages::Default,
            filter: CompileFilter::Default {
                required_features_filterable: false,
            },
            target_rustdoc_args: None,
            target_rustc_args: None,
            local_rustdoc_args: None,
            export_dir: None,
        },
        no_run: false,
        no_fail_fast: true,
    }
}
