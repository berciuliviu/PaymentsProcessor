use csv;

pub const TASKS_COUNT: u16 = 10;

// Declare const headers with lazy_static so allocation is possible at
// runtime https://docs.rs/lazy_static/latest/lazy_static/
lazy_static! {
    pub static ref FULL_HEADER: csv::ByteRecord =
        csv::ByteRecord::from(vec!["type", "client", "tx", "amount"]);
    pub static ref PARTIAL_HEADER: csv::ByteRecord =
        csv::ByteRecord::from(vec!["type", "client", "tx"]);
    pub static ref CSV_TOP_HEADER: csv::ByteRecord =
        csv::ByteRecord::from(vec!["client", "available", "held", "total", "locked"]);
}

pub fn create_csv_reader(filename: &String) -> csv::Reader<std::fs::File> {
    csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(filename)
        .unwrap_or_else(|err| {
            eprintln!("Error when trying to read from CSV: {}, {}", filename, err);
            std::process::exit(1);
        })
}
