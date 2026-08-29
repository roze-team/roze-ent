#![allow(dead_code)]

use roze_validation::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct CreateUserReq {
    #[validate(email, length(min = 1, max = 320))]
    pub email: String,
    #[validate(length(min = 1, max = 120))]
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct GetUserReq {
    #[validate(range(min = 1))]
    pub id: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct UserResp {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct ListUsersResp {
    pub users: Vec<UserResp>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct CreatePetReq {
    #[validate(range(min = 1))]
    pub owner_id: i64,
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    #[validate(length(min = 1))]
    pub species: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct GetPetReq {
    #[validate(range(min = 1))]
    pub id: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct PetResp {
    pub id: i64,
    pub owner_id: i64,
    pub name: String,
    pub species: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct ListPetsResp {
    pub pets: Vec<PetResp>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct DeleteResp {
    pub deleted: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct CreateGroupReq {
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct GetGroupReq {
    #[validate(range(min = 1))]
    pub id: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct GroupResp {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct ListGroupsResp {
    pub groups: Vec<GroupResp>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct MembershipPathReq {
    #[validate(range(min = 1))]
    pub group_id: i64,
    #[validate(range(min = 1))]
    pub user_id: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct UpdateMembershipRoleReq {
    #[validate(range(min = 1))]
    pub group_id: i64,
    #[validate(range(min = 1))]
    pub user_id: i64,
    #[validate(length(min = 1))]
    pub expected_role: String,
    #[validate(length(min = 1))]
    pub role: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct MembershipResp {
    pub id: i64,
    pub user_id: i64,
    pub group_id: i64,
    pub role: String,
    pub joined_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct EmptyReq {}
