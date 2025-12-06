pub mod query_log;
pub mod search_analytics;

pub use query_log::{QueryLog, SearchEvent, SearchClick};
pub use search_analytics::{
    SearchAnalytics, AnalyticsDashboard, PopularQuery, ZeroResultQuery,
    PeriodType, LatencyStats,
};
