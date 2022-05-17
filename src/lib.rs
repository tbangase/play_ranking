pub mod formatters;

use std::collections::HashMap;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use derive_new::new;
use getset::{Getters, Setters, CopyGetters};
use anyhow::Result;

use formatters::datetime_serde_format;

#[derive(new, Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Setters)]
pub struct PlayLog {
    player_id: String,
    #[getset(get_copy = "pub")]
    score: f64,
    #[getset(get = "pub")]
    #[serde(with = "datetime_serde_format")]
    create_timestamp: DateTime<Local>,
}

impl PlayLogExt for PlayLog {
    fn id(&self) -> String {
        self.player_id.clone()
    } 

    fn score(&self) -> f64 {
        self.score
    }

    fn set_score(&mut self, score: f64) -> &mut Self {
        self.score = score;
        self
    } 
}

pub trait PlayLogExt {
    fn id(&self) -> String;
    fn score(&self) -> f64;
    fn set_score(&mut self, score: f64) -> &mut Self;
}

/// This Trait is Implement For Vector of Playing. 
/// Usage as below:
/// ```
/// use ranking::*;
/// use chrono::Local;
/// 
/// let data = 
/// "create_timestamp,player_id,score
/// 2021/01/01 12:00,player0001,12345
/// 2021/01/02 13:00,player0002,10000
/// 2021/01/02 14:00,player0002,1800
/// ";
/// let mut reader = csv::Reader::from_reader(data.as_bytes());
/// let records = reader.deserialize().map(|record| {
///     let record: PlayLog = record.unwrap();
///     record
/// }).collect::<Vec<PlayLog>>();
/// assert_eq!(records.len(), 3);
/// let mean_records = records.mean();
/// assert_eq!(mean_records.len(), 2);
/// let top_player = mean_records.top_rankings(1).unwrap();
/// assert_eq!(top_player.len(), 1);
/// ```
pub trait Ranking<M: PlayLogExt> {
    // Return Ranking and PlayLog Structure
    fn mean(&self) -> Vec<M>;
    fn top_rankings(&self, top_rank: usize) -> Result<Vec<(usize, M)>>;
}

impl<M: PlayLogExt + Clone> Ranking<M> for Vec<M> {
    fn mean(&self) -> Vec<M> {
        let mut play_counts = HashMap::new();
        let mut res = vec![];

        for element in self.iter() {
            if let Some(target) = res.iter_mut().find(|players: &&mut M| players.id() == element.id()) {
                // Get Number of Play
                let k = *play_counts.entry(target.id()).or_insert(0) as f64;

                // Calc for Mean Score
                let term1 = target.score() / (k + 1.) * k;
                let term2 = element.score() / (k + 1.);

                target.set_score(term1 + term2);
                play_counts.entry(target.id()).and_modify(|count| *count += 1);
            } else {
                play_counts.insert(element.id(), 1);
                res.push((*element).clone());
            };
        }
        res
    }

    fn top_rankings(&self, top_rank: usize) -> Result<Vec<(usize, M)>> {
        let mut records = self.clone();

        // Sort by Score Descending
        records.sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap());

        let mut prev_score = f64::MAX;
        let mut prev_rank = 0;
        let mut res = vec![];

        for (i, element) in records.iter().enumerate() {
            let score = element.score().round();
            if i >= top_rank && prev_score != score { 
                break; 
            }
            let rank = if prev_score == score { prev_rank } else { i + 1 };

            res.push((rank, element.clone()));
            prev_score = score;
            prev_rank = rank;
        }
        Ok(res)
    }
}


#[cfg(test)]
mod play_log_test {
    use super::*;
    #[test]
    fn play_log_creation() {
        tracing_subscriber::fmt::init();

        let data = "
create_timestamp,player_id,score
2021/01/01 12:00,player0001,12345
2021/01/02 13:00,player0002,10000
2021/01/02 14:00,player0002,1800
";
        let mut reader = csv::Reader::from_reader(data.as_bytes());
        let playlogs = reader.deserialize().map(|record| {
            record.unwrap()
        }).collect::<Vec<PlayLog>>();
        let scores = vec![12345., 10000., 1800.];
        for (score, playlog) in scores.iter().zip(playlogs.iter()) {
            assert_eq!(playlog.score(), *score);
        }
        assert_eq!(playlogs.len(), 3);
    }

    #[test]
    fn play_log_top10() -> anyhow::Result<()> {
        let mut playlogs = vec![];
        for i in 0..20 {
            playlogs.push(PlayLog::new(format!("test{i}"), (100 * (i / 3)) as f64, Local::now()));
        }
        let top_10 = playlogs.top_rankings(10)?;
        let mut prev_rank = 0;
        for (rank, _log) in top_10.iter() {
            tracing::debug!("Rank: {rank}");
            assert!(*rank >= prev_rank);
            prev_rank = *rank;
        }
        assert!(top_10.len() >= 10);
        Ok(())
    }

    #[test]
    fn calc_mean() {
        //! Test Mean Score
        //! Granted for
        //! - Calculating Mean Score by sequentially
        let mut playlogs = vec![];
        for i in 1..=20 {
            playlogs.push(PlayLog::new(format!("test{}", i % 2), (100 * i) as f64, Local::now()));
        }
        let mean = playlogs.mean();
        assert_eq!(mean.len(), 2);
        assert_eq!(mean[0].score().round(), 1000.);
        assert_eq!(mean[1].score().round(), 1100.);
    }
}
