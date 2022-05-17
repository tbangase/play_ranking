use ranking::*;
use anyhow::Result;


fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let path = std::env::args().nth(1).expect("No CSV path is given.");

    let mut reader = csv::Reader::from_path(path).unwrap();
    let mut records: Vec<PlayLog> = vec![];
    for (i, record )in reader.deserialize().enumerate() {
        match record {
            Ok(record) => records.push(record),
            Err(err) => {
                tracing::error!("Fail to resolve Data of Line:{i}");
                tracing::error!("{err}");
                continue;
            }
        };
    }
    let records = records.mean();
    let top_10 = records.top_rankings(10)?;
    
    // println!("{}", serde_json::to_string_pretty(&top_10).unwrap());

    println!("rank,player_id,mean_score");
    for (rank, log) in top_10.iter() {
        println!("{rank},{},{}", log.id() ,log.score().round());
    }

    Ok(())
}