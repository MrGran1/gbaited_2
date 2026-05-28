use serde::{Deserialize, Serialize};

// ─── Users ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id:               String,
    pub visible_username: String,
    #[serde(skip_serializing)]
    pub password:         String,
    pub total_nbr_point:  u64,
}

// ─── Competitions ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Competition {
    pub id:        u64,
    pub game_name: String,
    pub region:    String,
}

// ─── Teams ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Team {
    pub id:   u64,
    pub name: String,
}

// ─── Matches ──────────────────────────────────────────────────────────────────

/// Raw row from `matchs` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MatchRow {
    pub id:             u64,
    pub score:          String,
    pub bo:             i16,   // aliased from `BO` in every query
    pub status:         String,
    pub competition_id: u64,
    pub team_1:         u64,
    pub team_2:         u64,
    pub winner:         u64,   // 0 = not decided yet
}

/// Enriched match returned by the API (replaces raw IDs with objects).
#[derive(Debug, Clone, Serialize)]
pub struct MatchDetail {
    pub id:             u64,
    pub score:          String,
    pub bo:             i16,
    pub status:         String,
    pub competition_id: u64,
    pub team_1:         Team,
    pub team_2:         Team,
    pub winner:         Option<Team>,
}

impl MatchDetail {
    pub fn from_row(row: MatchRow, t1: Team, t2: Team, winner: Option<Team>) -> Self {
        MatchDetail {
            id:             row.id,
            score:          row.score,
            bo:             row.bo,
            status:         row.status,
            competition_id: row.competition_id,
            team_1:         t1,
            team_2:         t2,
            winner,
        }
    }
}

// ─── Tournaments ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Tournament {
    pub id:               u64,
    pub tournament_name:  String,
    pub competition_id:   u64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TournamentAndUser {
    pub id:            u64,
    pub tournament_id: u64,
    pub user_id:       String,
}

// ─── Paris (bets) ─────────────────────────────────────────────────────────────

/// Raw bet row.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Pari {
    pub id:            u64,
    /// Format: "<winner_team_id>:<score>"  e.g. "7:3-1"
    pub prediction:    String,
    pub match_id:      u64,
    pub user_id:       String,
    pub tournament_id: u64,
}

// ─── Leaderboard (computed) ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    pub user_id:          String,
    pub visible_username: String,
    pub total_nbr_point:  u64,
    pub paris_count:      i64,
}
