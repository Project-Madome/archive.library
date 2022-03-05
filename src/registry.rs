pub use self::root_registry::RootRegistry;

mod root_registry {
    use sai::{combine_component_registry, component_registry, Component};

    use crate::{
        app::{HttpServer, Resolver},
        // command::CommandSet,
        config::Config,
        database::DatabaseSet,
        repository::{PostgresqlBookRepository, RepositorySet},
    };

    combine_component_registry!(
        RootRegistry,
        [
            ServerRegistry,
            ControllerRegistry,
            RepositoryRegistry,
            // CommandRegistry,
            ConfigRegistry
        ]
    );

    component_registry!(ServerRegistry, [HttpServer]);

    component_registry!(ControllerRegistry, [Resolver]);

    component_registry!(
        RepositoryRegistry,
        [DatabaseSet, RepositorySet, PostgresqlBookRepository]
    );

    // component_registry!(CommandRegistry, [CommandSet]);

    component_registry!(ConfigRegistry, [Config]);
}
