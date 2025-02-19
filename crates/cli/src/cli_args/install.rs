use crate::State;
use clap::Args;
use pacquet_package_manager::Install;
use pacquet_package_manifest::DependencyGroup;

#[derive(Debug, Args)]
pub struct InstallDependencyOptions {
    /// pacquet will not install any package listed in devDependencies and will remove those insofar
    /// they were already installed, if the NODE_ENV environment variable is set to production.
    /// Use this flag to instruct pacquet to ignore NODE_ENV and take its production status from this
    /// flag instead.
    #[arg(short = 'P', long)]
    prod: bool,
    /// Only devDependencies are installed and dependencies are removed insofar they were
    /// already installed, regardless of the NODE_ENV.
    #[arg(short = 'D', long)]
    dev: bool,
    /// optionalDependencies are not installed.
    #[arg(long)]
    no_optional: bool,
}

impl InstallDependencyOptions {
    /// Convert the dependency options to an iterator of [`DependencyGroup`]
    /// which filters the types of dependencies to install.
    fn dependency_groups(&self) -> impl Iterator<Item = DependencyGroup> {
        let &InstallDependencyOptions { prod, dev, no_optional } = self;
        let has_both = prod == dev;
        let has_prod = has_both || prod;
        let has_dev = has_both || dev;
        let has_optional = !no_optional;
        std::iter::empty()
            .chain(has_prod.then_some(DependencyGroup::Prod))
            .chain(has_dev.then_some(DependencyGroup::Dev))
            .chain(has_optional.then_some(DependencyGroup::Optional))
    }
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// --prod, --dev, and --no-optional
    #[clap(flatten)]
    pub dependency_options: InstallDependencyOptions,

    /// Don't generate a lockfile and fail if the lockfile is outdated.
    #[clap(long)]
    pub frozen_lockfile: bool,
}

impl InstallArgs {
    pub async fn run(self, state: State) -> miette::Result<()> {
        let State { tarball_mem_cache, http_client, config, manifest, lockfile } = &state;
        let InstallArgs { dependency_options, frozen_lockfile } = self;

        Install {
            tarball_mem_cache,
            http_client,
            config,
            manifest,
            lockfile: lockfile.as_ref(),
            dependency_groups: dependency_options.dependency_groups(),
            frozen_lockfile,
        }
        .run()
        .await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pacquet_package_manifest::DependencyGroup;
    use pretty_assertions::assert_eq;

    #[test]
    fn dependency_options_to_dependency_groups() {
        use DependencyGroup::{Dev, Optional, Prod};
        let create_list =
            |opts: InstallDependencyOptions| opts.dependency_groups().collect::<Vec<_>>();

        // no flags -> prod + dev + optional
        assert_eq!(
            create_list(InstallDependencyOptions { prod: false, dev: false, no_optional: false }),
            [Prod, Dev, Optional],
        );

        // --prod -> prod + optional
        assert_eq!(
            create_list(InstallDependencyOptions { prod: true, dev: false, no_optional: false }),
            [Prod, Optional],
        );

        // --dev -> dev + optional
        assert_eq!(
            create_list(InstallDependencyOptions { prod: false, dev: true, no_optional: false }),
            [Dev, Optional],
        );

        // --no-optional -> prod + dev
        assert_eq!(
            create_list(InstallDependencyOptions { prod: false, dev: false, no_optional: true }),
            [Prod, Dev],
        );

        // --prod --no-optional -> prod
        assert_eq!(
            create_list(InstallDependencyOptions { prod: true, dev: false, no_optional: true }),
            [Prod],
        );

        // --dev --no-optional -> dev
        assert_eq!(
            create_list(InstallDependencyOptions { prod: false, dev: true, no_optional: true }),
            [Dev],
        );

        // --prod --dev -> prod + dev + optional
        assert_eq!(
            create_list(InstallDependencyOptions { prod: true, dev: true, no_optional: false }),
            [Prod, Dev, Optional],
        );

        // --prod --dev --no-optional -> prod + dev
        assert_eq!(
            create_list(InstallDependencyOptions { prod: true, dev: true, no_optional: true }),
            [Prod, Dev],
        );
    }
}
