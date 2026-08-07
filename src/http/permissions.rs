use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, AsRefStr, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum GlobalRole {
    #[strum(serialize = "admin")]
    Admin,
    #[strum(serialize = "user")]
    User,
}

impl GlobalRole {
    pub fn get_permissions(&self) -> &[Permission] {
        match &self {
            GlobalRole::Admin => &*ADMIN_PERMISSIONS,
            GlobalRole::User => &*USER_PERMISSIONS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, AsRefStr)]
#[non_exhaustive]
pub enum Scope {
    #[strum(serialize = "owned")]
    Owned,
    #[strum(serialize = "partner_all")]
    PartnerAll,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScopeType {
    View,
    Write,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, AsRefStr)]
pub enum Permission {
    #[strum(serialize = "admin.analytics.view")]
    AdminAnalyticsView,
    #[strum(serialize = "admin.forecasts.view")]
    AdminForecastsView,
    #[strum(serialize = "admin.monitoring.view")]
    AdminMonitoringView,
    #[strum(serialize = "admin.partners.manage")]
    AdminPartnersManage,
    #[strum(serialize = "admin.partners.view")]
    AdminPartnersView,
    #[strum(serialize = "admin.system.manage")]
    AdminSystemManage,
    #[strum(serialize = "admin.system.view")]
    AdminSystemView,
    #[strum(serialize = "admin.users.manage")]
    AdminUsersManage,
    #[strum(serialize = "admin.users.view")]
    AdminUsersView,
    #[strum(serialize = "analytics.feedback.write")]
    AnalyticsFeedbackWrite,
    #[strum(serialize = "analytics.feedback.view")]
    AnalyticsFeedbackView,
    #[strum(serialize = "analytics.view")]
    AnalyticsView,
    #[strum(serialize = "cameras.write")]
    CamerasWrite,
    #[strum(serialize = "cameras.delete")]
    CamerasDelete,
    #[strum(serialize = "cameras.view")]
    CamerasView,
    #[strum(serialize = "feedback.write")]
    FeedbackWrite,
    #[strum(serialize = "forecasts.view")]
    ForecastsView,
    #[strum(serialize = "map.view")]
    MapView,
    #[strum(serialize = "occupancy.delete")]
    OccupancyDelete,
    #[strum(serialize = "occupancy.view")]
    OccupancyView,
    #[strum(serialize = "occupancy.write")]
    OccupancyWrite,
    #[strum(serialize = "partner_members.disable")]
    PartnerMembersDisable,
    #[strum(serialize = "partner_members.write")]
    PartnerMembersWrite,
    #[strum(serialize = "partner_members.view")]
    PartnerMembersView,
    #[strum(serialize = "partner_statistics.view")]
    PartnerStatisticsVIew,
    #[strum(serialize = "routing.write")]
    RoutingWrite,
    #[strum(serialize = "routing.delete")]
    RoutingDelete,
    #[strum(serialize = "routing.view")]
    RoutingView,
    #[strum(serialize = "sources.write")]
    SourcesWrite,
    #[strum(serialize = "sources.delete")]
    SourcesDelete,
    #[strum(serialize = "sources.view")]
    SourcesView,
    #[strum(serialize = "users.me.manage")]
    UsersMeManage,
    #[strum(serialize = "zones.write")]
    ZonesWrite,
    #[strum(serialize = "zones.delete")]
    ZonesDelete,
    #[strum(serialize = "zones.view")]
    ZonesView,
}

static USER_PERMISSIONS: LazyLock<Vec<Permission>> = LazyLock::new(|| {
    vec![
        Permission::UsersMeManage,
        Permission::MapView,
        Permission::ZonesView,
        Permission::OccupancyView,
        Permission::ForecastsView,
        Permission::SourcesView,
        Permission::RoutingView,
        Permission::RoutingWrite,
        Permission::RoutingDelete,
        Permission::FeedbackWrite,
    ]
});

static ADMIN_PERMISSIONS: LazyLock<Vec<Permission>> = LazyLock::new(|| {
    [
        USER_PERMISSIONS.clone(),
        vec![
            Permission::AdminUsersView,
            Permission::AdminUsersManage,
            Permission::AdminPartnersView,
            Permission::AdminPartnersManage,
            Permission::AdminSystemView,
            Permission::AdminSystemManage,
            Permission::AdminMonitoringView,
            Permission::AdminAnalyticsView,
            Permission::AdminForecastsView,
            Permission::AnalyticsView,
            Permission::AnalyticsFeedbackWrite,
            Permission::AnalyticsFeedbackView,
            Permission::CamerasView,
            Permission::CamerasWrite,
            Permission::CamerasDelete,
            Permission::ZonesView,
            Permission::ZonesWrite,
            Permission::ZonesDelete,
            Permission::PartnerMembersView,
            Permission::PartnerMembersWrite,
            Permission::PartnerMembersDisable,
        ],
    ]
    .concat()
});
