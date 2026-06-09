mod api;
mod cli;
mod client;
mod display;
mod gpt;
mod models;
mod persistence;
fn main() {
    // 谦者，众善之基；傲者，众恶之魁。
    // Humility is the foundation of all good; pride is the leader of all evil.
    // ------------------------------------------------------------------------
    env_logger::init();
    cli::ArgParse::parse();
}
