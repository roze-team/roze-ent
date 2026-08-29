// Application-owned logic prelude.
//
// Declare shared helper modules, imports, and re-exports in this file.
// Its contents are included at the `logic` module level.
// `rozectl ... generate --update` preserves it while rebuilding `logic/mod.rs`.

pub(crate) fn user_response(model: crate::model::UserModel) -> UserResp {
    UserResp {
        id: model.id,
        email: model.email,
        name: model.name,
        active: model.active,
        created_at: model.created_at,
    }
}

pub(crate) fn pet_response(model: crate::model::PetModel) -> PetResp {
    PetResp {
        id: model.id,
        owner_id: model.owner_id,
        name: model.name,
        species: model.species,
    }
}

pub(crate) fn group_response(model: crate::model::GroupModel) -> GroupResp {
    GroupResp {
        id: model.id,
        name: model.name,
        description: model.description,
        created_at: model.created_at,
    }
}

pub(crate) fn membership_response(model: crate::model::MembershipModel) -> MembershipResp {
    MembershipResp {
        id: model.id,
        user_id: model.user_id,
        group_id: model.group_id,
        role: model.role,
        joined_at: model.joined_at,
    }
}

pub(crate) fn project_response(model: crate::model::ProjectModel) -> ProjectResp {
    ProjectResp {
        id: model.id,
        tenant_id: model.tenant_id,
        name: model.name,
        description: model.description,
        version: model.version,
        deleted_at: model.deleted_at,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

pub(crate) fn model_error(error: anyhow::Error) -> RozeError {
    tracing::error!(error = %error, "entity model operation failed");
    RozeError::Internal("entity model operation failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_user_model_to_wire_response() {
        let response = user_response(crate::model::UserModel {
            id: 7,
            email: "alice@example.com".to_string(),
            name: "Alice".to_string(),
            active: true,
            created_at: 42,
            manager_id: None,
        });

        assert_eq!(response.id, 7);
        assert_eq!(response.email, "alice@example.com");
        assert_eq!(response.name, "Alice");
        assert!(response.active);
        assert_eq!(response.created_at, 42);
    }

    #[test]
    fn maps_pet_model_to_wire_response() {
        let response = pet_response(crate::model::PetModel {
            id: 9,
            owner_id: 7,
            name: "Milo".to_string(),
            species: "cat".to_string(),
        });

        assert_eq!(response.id, 9);
        assert_eq!(response.owner_id, 7);
        assert_eq!(response.name, "Milo");
        assert_eq!(response.species, "cat");
    }

    #[test]
    fn maps_group_model_to_wire_response() {
        let response = group_response(crate::model::GroupModel {
            id: 11,
            name: "rustaceans".to_string(),
            description: Some("Rust users".to_string()),
            created_at: 77,
        });

        assert_eq!(response.id, 11);
        assert_eq!(response.name, "rustaceans");
        assert_eq!(response.description.as_deref(), Some("Rust users"));
        assert_eq!(response.created_at, 77);
    }

    #[test]
    fn maps_membership_model_to_wire_response() {
        let response = membership_response(crate::model::MembershipModel {
            id: 13,
            user_id: 7,
            group_id: 11,
            role: "admin".to_string(),
            joined_at: 99,
        });

        assert_eq!(response.id, 13);
        assert_eq!(response.user_id, 7);
        assert_eq!(response.group_id, 11);
        assert_eq!(response.role, "admin");
        assert_eq!(response.joined_at, 99);
    }

    #[test]
    fn maps_project_model_to_wire_response() {
        let response = project_response(crate::model::ProjectModel {
            id: 17,
            tenant_id: "tenant-a".to_string(),
            name: "compiler".to_string(),
            description: None,
            version: 3,
            deleted_at: None,
            created_at: 101,
            updated_at: 202,
        });

        assert_eq!(response.id, 17);
        assert_eq!(response.tenant_id, "tenant-a");
        assert_eq!(response.name, "compiler");
        assert_eq!(response.version, 3);
        assert_eq!(response.deleted_at, None);
        assert_eq!(response.created_at, 101);
        assert_eq!(response.updated_at, 202);
    }
}
