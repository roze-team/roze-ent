use std::{collections::BTreeMap, fs, path::Path};

use serde_json::Value;

const EXPECTED_PERMISSIONS: &[(&str, &str)] = &[
    ("addGroupMember", "groups:write"),
    ("createGroup", "groups:write"),
    ("createPet", "pets:write"),
    ("createProject", "projects:write"),
    ("createUser", "users:write"),
    ("deletePet", "pets:write"),
    ("deleteProject", "projects:write"),
    ("deleteUser", "users:write"),
    ("getGroup", "groups:read"),
    ("getPet", "pets:read"),
    ("getProject", "projects:read"),
    ("getUser", "users:read"),
    ("listGroupUsers", "groups:read"),
    ("listGroups", "groups:read"),
    ("listProjects", "projects:read"),
    ("listUserGroups", "groups:read"),
    ("listUserPets", "pets:read"),
    ("listUsers", "users:read"),
    ("removeGroupMember", "groups:write"),
    ("updateGroupMember", "groups:write"),
    ("updateProject", "projects:write"),
];

#[test]
fn every_business_operation_declares_its_required_permission() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/openapi.json");
    let document: Value = serde_json::from_slice(
        &fs::read(path).expect("generated OpenAPI document must be readable"),
    )
    .expect("generated OpenAPI document must be valid JSON");

    let mut actual = BTreeMap::new();
    for path_item in document["paths"]
        .as_object()
        .expect("OpenAPI paths must be an object")
        .values()
    {
        for operation in path_item
            .as_object()
            .expect("OpenAPI path item must be an object")
            .values()
        {
            let is_business_operation = operation["tags"]
                .as_array()
                .is_some_and(|tags| tags.iter().any(|tag| tag == "roze-ent"));
            if !is_business_operation {
                continue;
            }

            let operation_id = operation["operationId"]
                .as_str()
                .expect("business operation must have an operationId");
            let permissions = operation["x-roze-permissions"]
                .as_array()
                .expect("business operation must declare permissions");
            assert_eq!(
                permissions.len(),
                1,
                "{operation_id} must require exactly one permission"
            );
            let permission = permissions[0]
                .as_str()
                .expect("permission must be a string");
            assert!(
                actual.insert(operation_id, permission).is_none(),
                "duplicate operationId: {operation_id}"
            );
        }
    }

    let expected: BTreeMap<_, _> = EXPECTED_PERMISSIONS.iter().copied().collect();
    assert_eq!(actual, expected);
}
