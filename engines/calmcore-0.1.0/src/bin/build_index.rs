use std::env;
use std::io::BufRead;
use std::time::Instant;

use calmcore;
use calmcore::{Action, ActionType, CalmCore, Config};
use calmcore::util::CoreResult;
use proto::core::field::{FulltextOption, TermOption, Type};
use proto::core::field::fulltext_option::Tokenizer;
use proto::core::field::Option::Fulltext;
use proto::core::field::Option::Term;
use proto::core::Schema;

fn main() {
    let args: Vec<String> = env::args().collect();
    main_inner("calmcore-bench", &args[1]).unwrap();
}

fn main_inner(schema_name: &str, data_path: &str) -> CoreResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let schema = make_schema(schema_name);
    let core = CalmCore::new_with_conf(Config {
        data_path: data_path.to_string(),
        segment_max_size: 500_000,
        flush_interval_secs: 300,
    })?;
    let space = core.create_engine(schema)?;

    println!("Starting data insertion...");
    let start = Instant::now();

    let batch_size = 1000;
    let mut total = 0;
    let mut actions = Vec::with_capacity(batch_size);

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        total += 1;
        actions.push(Action::new(ActionType::Append, "", line.as_bytes()));

        if total % 1000 == 0 {
            space.mutate(actions, None)?;
            actions = Vec::with_capacity(batch_size);
        }

        if total % 100_000 == 0 {
            println!("{}", total);

            if space.segment_readers().iter().filter(|r| r.is_hot()).count() > 3 {
                println!("Too many hot segments, waiting for compaction...");
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    }

    if !actions.is_empty() {
        space.mutate(actions, None)?;
    }

    space.persist()?;

    println!("Data insertion finished in {:?}", start.elapsed());

    Ok(())
}

fn make_schema(schema_name: &str) -> Schema {
    calmcore::easy_schema(
        schema_name,
        vec![
            ("id".to_string(), Type::String, Some(Term(TermOption { no_index: false, no_store: false }))),
            ("text".to_string(), Type::Text, Some(Fulltext(FulltextOption {
                tokenizer: Tokenizer::Standard as i32,
                filters: Vec::new(),
                stopwords: None,
                synonyms: None,
                no_store: true,
            }))),
        ],
    )
}
