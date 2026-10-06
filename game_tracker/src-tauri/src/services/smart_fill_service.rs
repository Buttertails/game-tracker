use rand::seq::IndexedRandom;

use crate::db::game_entries;
use crate::models::error::AppError;
use crate::models::{Era, GameEntry, LengthCategory, ShelfEntries, derive_era, derive_length_category};
use crate::services::status_transition_service::StatusTransitionService;
use crate::{db::Database, models::SmartFillSuggestion};

#[derive(Debug, Clone)]
pub struct DiversityProfile {
    pub genre: String,
    pub length_category: LengthCategory,
    pub era: Era,
}

pub struct SmartFillService<'a> {
    db: &'a Database,
}

impl<'a> SmartFillService<'a> {
    pub fn new(db: &'a Database) -> Self {
        SmartFillService { db }
    }

    pub fn get_suggestions(&self, shelf_id: i64) -> Result<Vec<SmartFillSuggestion>, AppError> {
        let slots_available = 3 - game_entries::count_in_progress(self.db, shelf_id)? as usize;
        if slots_available == 0 {
            return Err(AppError::SmartFillNoSlots(
                "In Progress is full. Can not fill.".to_string(),
            ));
        }

        let shelf_entries = game_entries::list_entries_by_shelf(self.db, shelf_id)?;
        let eligble_backlog = self.get_eligible_backlog(&shelf_entries)?;

        if eligble_backlog.is_empty() {
            return Err(AppError::SmartFillNotEligible(
                "No eligible games for SmartFill".to_string(),
            ));
        }

        let mut diversity_context = self.build_diversity_context(&shelf_entries);

        let selected =
            self.select_suggestions(eligble_backlog, &mut diversity_context, slots_available);

        let suggestions = selected
            .iter()
            .map(|entry| SmartFillSuggestion {
                entry_id: entry.entry_id,
                name: entry.name.clone(),
                genre: entry.genre.clone().unwrap(),
                length_category: derive_length_category(entry.avg_playtime_hours.unwrap()),
                era: derive_era(entry.release_date.unwrap()),
                diversity_score: self.score_candidate(entry, &diversity_context),
                background_image: entry.background_image.clone(),
            })
            .collect();

        Ok(suggestions)
    }

    pub fn accept(&self, entry_ids: &[i64]) -> Result<(), AppError> {
        let transition_service = StatusTransitionService::new(self.db);

        for &id in entry_ids {
            transition_service.move_to_in_progress(id)?;
        }

        Ok(())
    }

    pub fn reroll(
        &self,
        shelf_id: i64,
        entry_to_replace_id: i64,
        kept_suggestion_ids: &[i64],
    ) -> Result<SmartFillSuggestion, AppError> {
        let shelf_entries = game_entries::list_entries_by_shelf(self.db, shelf_id)?;

        let mut context = self.build_diversity_context(&shelf_entries);

        for &id in kept_suggestion_ids {
            let entry = game_entries::get_entry(self.db, id)?;
            context.push(DiversityProfile {
                genre: entry.genre.clone().unwrap(),
                length_category: derive_length_category(entry.avg_playtime_hours.unwrap()),
                era: derive_era(entry.release_date.unwrap()),
            });
        }

        let candidates: Vec<GameEntry> = self
            .get_eligible_backlog(&shelf_entries)?
            .into_iter()
            .filter(|e| {
                e.entry_id != entry_to_replace_id && !kept_suggestion_ids.contains(&e.entry_id)
            })
            .collect();

        if candidates.is_empty() {
            return Err(AppError::SmartFillNotEligible(
                "No other eligible entries".to_string(),
            ));
        }

        let selected = self.select_suggestions(candidates, &mut context, 1);

        let entry = &selected[0];
        Ok(SmartFillSuggestion {
            entry_id: entry.entry_id,
            name: entry.name.clone(),
            genre: entry.genre.clone().unwrap(),
            length_category: derive_length_category(entry.avg_playtime_hours.unwrap()),
            era: derive_era(entry.release_date.unwrap()),
            diversity_score: self.score_candidate(entry, &context),
            background_image: entry.background_image.clone(),
        })
    }

    fn select_suggestions(
        &self,
        mut candidates: Vec<GameEntry>,
        context: &mut Vec<DiversityProfile>,
        slots: usize,
    ) -> Vec<GameEntry> {
        let mut selected: Vec<GameEntry> = Vec::new();

        for _ in 0..slots {
            if candidates.is_empty() {
                break;
            }

            let max_score = candidates
                .iter()
                .map(|c| self.score_candidate(c, context))
                .max()
                .unwrap_or(0);

            let best_indices: Vec<usize> = candidates
                .iter()
                .enumerate()
                .filter(|(_, c)| self.score_candidate(c, context) == max_score)
                .map(|(i, _)| i)
                .collect();

            let &chosen_idx = best_indices.choose(&mut rand::rng()).unwrap();

            let chosen = candidates.remove(chosen_idx);

            context.push(DiversityProfile {
                genre: chosen.genre.clone().unwrap(),
                length_category: derive_length_category(chosen.avg_playtime_hours.unwrap()),
                era: derive_era(chosen.release_date.unwrap()),
            });

            selected.push(chosen);
        }

        selected
    }

    fn score_candidate(&self, candidate: &GameEntry, context: &[DiversityProfile]) -> u32 {
        let mut score: u32 = 0;

        let cand_genre = candidate.genre.as_ref().unwrap();
        let cand_length = derive_length_category(candidate.avg_playtime_hours.unwrap());
        let cand_era = derive_era(candidate.release_date.unwrap());

        for ctx in context {
            if cand_genre != &ctx.genre {
                score += 1;
            }
            if cand_length != ctx.length_category {
                score += 1;
            }
            if cand_era != ctx.era {
                score += 1;
            }
        }
        score
    }

    fn build_diversity_context(&self, shelf_entries: &ShelfEntries) -> Vec<DiversityProfile> {
        let diversity_profile = shelf_entries
            .in_progress
            .iter()
            .filter_map(|ge| {
                let genre = ge.genre.as_ref()?.clone();
                let length_category = derive_length_category(ge.avg_playtime_hours?);
                let era = derive_era(ge.release_date?);

                Some(DiversityProfile {
                    genre,
                    length_category,
                    era,
                })
            })
            .collect();

        diversity_profile
    }

    pub fn get_eligible_backlog(
        &self,
        shelf_entries: &ShelfEntries,
    ) -> Result<Vec<GameEntry>, AppError> {
        let eligible_backlog = shelf_entries
            .backlog
            .iter()
            .filter(|ge| {
                ge.genre.is_some() && ge.avg_playtime_hours.is_some() && ge.release_date.is_some()
            })
            .cloned()
            .collect::<Vec<GameEntry>>();
        Ok(eligible_backlog)
    }
}
