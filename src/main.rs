use csv::Writer;
use ignore::WalkBuilder;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: todo_finder <directory> <output.csv>");
        std::process::exit(1);
    }

    let directory = &args[1];
    let output_file = &args[2];

    let mut csv_writer = Writer::from_path(output_file)?;
    csv_writer.write_record(&["File", "Line", "Comment"])?;

    let walker = WalkBuilder::new(directory)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker
        .filter_map(|e| e.ok())
        .filter(todo::utils::is_supported_file)
    {
        todo::utils::process_file(entry.path(), &mut csv_writer)?;
    }

    csv_writer.flush()?;

    println!("Results saved to: {}", output_file);

    Ok(())
}
