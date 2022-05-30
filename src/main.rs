use std::env;
// use toy_processor::processor::Processor;
use toy_processor::processor_async::ProcessorAsync;

// fn main() {
//     // Process arguments
//     let args: Vec<String> = env::args().collect();

//     if args.len() > 2 {
//         eprintln!("There should be only one argument given to the program.");
//         std::process::exit(1);
//     }

//     let filename: String = match args.get(1) {
//         Some(file) => file.to_string(),
//         None => {
//             eprintln!("Error! No argument provided.");
//             std::process::exit(1);
//         }
//     };

//     if !std::path::Path::new(&filename).exists() {
//         eprintln!("File {} does not exist.", filename);
//         std::process::exit(1);
//     }

//     let mut toy_processor: Processor = Processor::new(filename);

//     toy_processor.process_transactions();

//     if let Err(error) = toy_processor.print_clients(true) {
//         eprintln!("{}", error);
//     }
// }

#[tokio::main]
async fn main() {
    // Process arguments
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        eprintln!("There should be only one argument given to the program.");
        std::process::exit(1);
    }

    let filename: String = match args.get(1) {
        Some(file) => file.to_string(),
        None => {
            eprintln!("Error! No argument provided.");
            std::process::exit(1);
        }
    };

    if !std::path::Path::new(&filename).exists() {
        eprintln!("File {} does not exist.", filename);
        std::process::exit(1);
    }

    let mut toy_processor: ProcessorAsync = ProcessorAsync::new(filename);

    toy_processor.process_transactions_async().await;
}
