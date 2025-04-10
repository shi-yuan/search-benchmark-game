use std::env;
use std::io::BufRead;

use calmcore::CalmCore;
use calmcore::util::CoreResult;

fn main() {
    let args: Vec<String> = env::args().collect();
    main_inner("calmcore-bench", &args[1]).unwrap()
}

fn main_inner(schema_name: &str, data_path: &str) -> CoreResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let sql = format!("select * from {} where", schema_name);

    let core = CalmCore::new(data_path)?;
    let engine = core.load_engine(schema_name)?;

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let fields: Vec<&str> = line.split("\t").collect();
        assert_eq!(fields.len(), 2, "Expected a line in the format <COMMAND> query.");

        let command = fields[0];
        let query = parse_query(fields[1]);
        let count;
        match command {
            "COUNT" => {
                count = engine.sql(format!("{sql} {query} limit 0").as_str())?.total_hits;
            }
            "TOP_10" => {
                let _top_k = engine.sql(format!("{sql} {query} limit 10").as_str())?;
                count = 1;
            }
            "TOP_100" => {
                let _top_k = engine.sql(format!("{sql} {query} limit 100").as_str())?;
                count = 1;
            }
            "TOP_1000" => {
                let _top_k = engine.sql(format!("{sql} {query} limit 1000").as_str())?;
                count = 1;
            }
            "TOP_1_COUNT" => {
                count = engine.sql(format!("{sql} {query} limit 1").as_str())?.total_hits;
            }
            "TOP_5_COUNT" => {
                count = engine.sql(format!("{sql} {query} limit 5").as_str())?.total_hits;
            }
            "TOP_10_COUNT" => {
                count = engine.sql(format!("{sql} {query} limit 10").as_str())?.total_hits;
            }
            "TOP_100_COUNT" => {
                count = engine.sql(format!("{sql} {query} limit 100").as_str())?.total_hits;
            }
            "TOP_1000_COUNT" => {
                count = engine.sql(format!("{sql} {query} limit 1000").as_str())?.total_hits;
            }
            _ => {
                println!("UNSUPPORTED");
                continue;
            }
        }
        println!("{}", count);
    }

    Ok(())
}

fn parse_query(query: &str) -> String {
    if query.contains('"') {
        if query.contains('+') {
            // tags:two-phase-critic: +"the who" +uk
            query.split('+')
                .filter(|e| !e.trim().is_empty())
                .map(|e| {
                    let s = e.trim();
                    if s.contains('"') {
                        format!("text=phrase('{}',slop=0)", s.replace('"', ""))
                    } else {
                        format!("text='{}'", s)
                    }
                })
                .collect::<Vec<_>>().join(" and ")
        } else {
            // tags:phrase
            format!("text=phrase('{}',slop=0)", query.replace('"', ""))
        }
    } else {
        if query.contains('+') {
            // tags:intersection
            format!("text=text('{}',operator='and')", query.replace('+', ""))
        } else {
            // tags:union,term
            format!("text=text('{}',operator='or')", query)
        }
    }
}
