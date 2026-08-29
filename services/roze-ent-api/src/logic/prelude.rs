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
}
