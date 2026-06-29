fn main() {
    tracing_subscriber::fmt::init();
    finance_app::run().unwrap()
}
