use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::AssertSqlSafe;

use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::handlers::activities::{Activity, SELECT_ACTIVITY};
use crate::handlers::deals::STAGES;
use crate::state::AppState;

#[derive(Serialize)]
pub struct Dashboard {
    pub stats: Stats,
    pub monthly_sales: Vec<MonthlySales>,
    pub pipeline: Vec<StageTotal>,
    pub recent_activities: Vec<Activity>,
    pub upcoming_follow_ups: Vec<Activity>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Stats {
    pub deals_won: i64,
    pub deals_lost: i64,
    pub new_leads_this_month: i64,
    pub new_leads_last_month: i64,
    pub active_deals: i64,
    pub pipeline_value: f64,
    pub won_revenue: f64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct MonthlySales {
    pub month: String,
    pub revenue: f64,
    pub deals: i64,
}

#[derive(Serialize)]
pub struct StageTotal {
    pub stage: String,
    pub count: i64,
    pub value: f64,
}

pub async fn summary(State(state): State<AppState>, _user: AuthUser) -> AppResult<Json<Dashboard>> {
    let stats = sqlx::query_as::<_, Stats>(
        "SELECT
            (SELECT COUNT(*) FROM deals WHERE stage = 'Won')  AS deals_won,
            (SELECT COUNT(*) FROM deals WHERE stage = 'Lost') AS deals_lost,
            (SELECT COUNT(*) FROM leads
                WHERE created_at >= date_trunc('month', NOW())) AS new_leads_this_month,
            (SELECT COUNT(*) FROM leads
                WHERE created_at >= date_trunc('month', NOW()) - INTERVAL '1 month'
                  AND created_at <  date_trunc('month', NOW())) AS new_leads_last_month,
            (SELECT COUNT(*) FROM deals WHERE stage NOT IN ('Won', 'Lost')) AS active_deals,
            (SELECT COALESCE(SUM(amount), 0) FROM deals
                WHERE stage NOT IN ('Won', 'Lost')) AS pipeline_value,
            (SELECT COALESCE(SUM(amount), 0) FROM deals WHERE stage = 'Won') AS won_revenue",
    )
        .fetch_one(&state.db)
        .await?;

    let monthly_sales = sqlx::query_as::<_, MonthlySales>(
        "SELECT to_char(m, 'Mon') AS month,
                COALESCE(SUM(d.amount), 0) AS revenue,
                COUNT(d.id) AS deals
         FROM generate_series(
                date_trunc('month', NOW()) - INTERVAL '5 months',
                date_trunc('month', NOW()),
                INTERVAL '1 month') AS m
         LEFT JOIN deals d
                ON d.stage = 'Won'
               AND date_trunc('month', COALESCE(d.expected_close::timestamptz, d.created_at)) = m
         GROUP BY m
         ORDER BY m",
    )
        .fetch_all(&state.db)
        .await?;

    let rows = sqlx::query_as::<_, (String, i64, f64)>(
        "SELECT stage, COUNT(*), COALESCE(SUM(amount), 0) FROM deals GROUP BY stage",
    )
        .fetch_all(&state.db)
        .await?;
    let pipeline = STAGES
        .iter()
        .map(|stage| {
            let (count, value) = rows
                .iter()
                .find(|(s, _, _)| s == stage)
                .map(|(_, c, v)| (*c, *v))
                .unwrap_or((0, 0.0));
            StageTotal { stage: stage.to_string(), count, value }
        })
        .collect();

    let recent_activities = sqlx::query_as::<_, Activity>(AssertSqlSafe(format!(
        "{SELECT_ACTIVITY}
         WHERE a.status = 'Completed'
         ORDER BY COALESCE(a.due_at, a.created_at) DESC
         LIMIT 5"
    )))
        .fetch_all(&state.db)
        .await?;

    let upcoming_follow_ups = sqlx::query_as::<_, Activity>(AssertSqlSafe(format!(
        "{SELECT_ACTIVITY}
         WHERE a.status = 'Pending' AND a.due_at >= NOW()
         ORDER BY a.due_at
         LIMIT 5"
    )))
        .fetch_all(&state.db)
        .await?;

    Ok(Json(Dashboard {
        stats,
        monthly_sales,
        pipeline,
        recent_activities,
        upcoming_follow_ups,
    }))
}