use std::str::FromStr;

use async_graphql::{ComplexObject, Context, Enum, SimpleObject};
use chrono::{DateTime, Utc};

use async_graphql::dataloader::DataLoader;
use uuid::Uuid;

use super::{labels::Label, member::Member, project::Project};

use super::loaders::{LabelLoader, MemberLoader, ProjectLoader};
use crate::{auth::core::PlexoAuthToken, system::core::Engine};
#[derive(SimpleObject, Clone, Debug)]
#[graphql(complex)]
pub struct Task {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub title: String,
    pub description: Option<String>,

    pub owner_id: Uuid,

    pub status: TaskStatus,
    pub priority: TaskPriority,

    pub due_date: Option<DateTime<Utc>>,

    pub project_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,

    pub count: i32,
}

#[ComplexObject]
impl Task {
    pub async fn owner(&self, ctx: &Context<'_>) -> Option<Member> {
        let loader = ctx.data::<DataLoader<MemberLoader>>().unwrap();

        //match to see is project_id is none
        loader.load_one(self.owner_id).await.unwrap()
    }

    pub async fn leader(&self, ctx: &Context<'_>) -> Option<Member> {
        let loader = ctx.data::<DataLoader<MemberLoader>>().unwrap();

        //match to see is project_id is none
        match self.lead_id {
            Some(lead_id) => loader.load_one(lead_id).await.unwrap(),
            None => None,
        }
    }

    pub async fn project(&self, ctx: &Context<'_>) -> Option<Project> {
        let loader = ctx.data::<DataLoader<ProjectLoader>>().unwrap();

        //match to see is project_id is none
        match self.project_id {
            Some(project_id) => loader.load_one(project_id).await.unwrap(),
            None => None,
        }
    }

    pub async fn assignees(&self, ctx: &Context<'_>) -> Vec<Member> {
        let auth_token = &ctx.data::<PlexoAuthToken>().unwrap().0;
        let plexo_engine = ctx.data::<Engine>().unwrap();
        println!("token: {}", auth_token);

        let loader = ctx.data::<DataLoader<MemberLoader>>().unwrap();

        let ids: Vec<Uuid> = sqlx::query!(
            r#"
            SELECT assignee_id FROM tasks_by_assignees
            WHERE task_id = $1
            "#,
            &self.id
        )
        .fetch_all(&*plexo_engine.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|id| id.assignee_id)
        .collect();

        let members_map = loader.load_many(ids.clone()).await.unwrap();

        let members: &Vec<Member> = &ids
            .into_iter()
            .map(|id| members_map.get(&id).unwrap().clone())
            .collect();

        members.clone()
    }

    pub async fn labels(&self, ctx: &Context<'_>) -> Vec<Label> {
        let auth_token = &ctx.data::<PlexoAuthToken>().unwrap().0;
        let plexo_engine = ctx.data::<Engine>().unwrap();
        println!("token: {}", auth_token);

        let loader = ctx.data::<DataLoader<LabelLoader>>().unwrap();

        let ids: Vec<Uuid> = sqlx::query!(
            r#"
            SELECT label_id FROM labels_by_tasks
            WHERE task_id = $1
            "#,
            &self.id
        )
        .fetch_all(&*plexo_engine.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|id| id.label_id)
        .collect();

        let labels_map = loader.load_many(ids.clone()).await.unwrap();

        let labels: &Vec<Label> = &ids
            .into_iter()
            .map(|id| labels_map.get(&id).unwrap().clone())
            .collect();

        labels.clone()
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum TaskStatus {
    None,
    Backlog,
    ToDo,
    InProgress,
    Done,
    Canceled,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum TaskPriority {
    None,
    Low,
    Medium,
    High,
    Urgent,
}

impl TaskStatus {
    pub fn from_optional_str(s: &Option<String>) -> Self {
        match s {
            Some(s) => Self::from_str(s.as_str()).unwrap_or(Self::None),
            None => Self::None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Backlog => "Backlog",
            Self::ToDo => "ToDo",
            Self::InProgress => "InProgress",
            Self::Done => "Done",
            Self::Canceled => "Canceled",
        }
    }
}

impl FromStr for TaskStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" => Ok(Self::None),
            "Backlog" => Ok(Self::Backlog),
            "ToDo" => Ok(Self::ToDo),
            "InProgress" => Ok(Self::InProgress),
            "Done" => Ok(Self::Done),
            "Canceled" => Ok(Self::Canceled),
            _ => Err(()),
        }
    }
}

impl TaskPriority {
    pub fn from_optional_str(s: &Option<String>) -> Self {
        match s {
            Some(s) => Self::from_str(s.as_str()).unwrap_or(Self::None),
            None => Self::None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Urgent => "Urgent",
        }
    }
}

impl FromStr for TaskPriority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" => Ok(Self::None),
            "Low" => Ok(Self::Low),
            "Medium" => Ok(Self::Medium),
            "High" => Ok(Self::High),
            "Urgent" => Ok(Self::Urgent),
            _ => Err(()),
        }
    }
}
