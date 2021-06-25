use postgres::{Client, NoTls};

struct Globals {
    pub postgres: Client,
}

pub const GLOBALS: Globals = Globals {
    postgres: Client::connect(
        "host=localhost user=admin password=123 dbname=maldness_bot",
        NoTls,
    )
    .expect("Failed to connect to postgres"),
};
