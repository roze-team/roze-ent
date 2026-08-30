use std::collections::BTreeMap;

use roze_openapi::{HttpMethod, OpenApiBuilder, Operation, Schema};

pub fn document() -> serde_json::Value {
    let mut builder =
        OpenApiBuilder::new("roze-ent", "0.1.0").description("service group: entities");
    builder = builder.server("/api/v1", "service: roze-ent");
    {
        let mut properties = BTreeMap::new();
        properties.insert("email".to_string(), Schema::string());
        properties.insert("name".to_string(), Schema::string());
        builder = builder.component_schema(
            "CreateUserReq",
            Schema::object(properties, vec!["email".to_string(), "name".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        builder = builder.component_schema(
            "GetUserReq",
            Schema::object(properties, vec!["id".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        properties.insert("email".to_string(), Schema::string());
        properties.insert("name".to_string(), Schema::string());
        properties.insert("active".to_string(), Schema::boolean());
        properties.insert("created_at".to_string(), Schema::integer("int64"));
        builder = builder.component_schema(
            "UserResp",
            Schema::object(
                properties,
                vec![
                    "id".to_string(),
                    "email".to_string(),
                    "name".to_string(),
                    "active".to_string(),
                    "created_at".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert(
            "users".to_string(),
            Schema::array(Schema::reference("UserResp")),
        );
        builder = builder.component_schema(
            "ListUsersResp",
            Schema::object(properties, vec!["users".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("owner_id".to_string(), Schema::integer("int64"));
        properties.insert("name".to_string(), Schema::string());
        properties.insert("species".to_string(), Schema::string());
        builder = builder.component_schema(
            "CreatePetReq",
            Schema::object(
                properties,
                vec![
                    "owner_id".to_string(),
                    "name".to_string(),
                    "species".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        builder = builder.component_schema(
            "GetPetReq",
            Schema::object(properties, vec!["id".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        properties.insert("owner_id".to_string(), Schema::integer("int64"));
        properties.insert("name".to_string(), Schema::string());
        properties.insert("species".to_string(), Schema::string());
        builder = builder.component_schema(
            "PetResp",
            Schema::object(
                properties,
                vec![
                    "id".to_string(),
                    "owner_id".to_string(),
                    "name".to_string(),
                    "species".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert(
            "pets".to_string(),
            Schema::array(Schema::reference("PetResp")),
        );
        builder = builder.component_schema(
            "ListPetsResp",
            Schema::object(properties, vec!["pets".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("deleted".to_string(), Schema::boolean());
        builder = builder.component_schema(
            "DeleteResp",
            Schema::object(properties, vec!["deleted".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("name".to_string(), Schema::string());
        properties.insert(
            "description".to_string(),
            Schema::reference("Option<String>"),
        );
        builder = builder.component_schema(
            "CreateGroupReq",
            Schema::object(properties, vec!["name".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        builder = builder.component_schema(
            "GetGroupReq",
            Schema::object(properties, vec!["id".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        properties.insert("name".to_string(), Schema::string());
        properties.insert(
            "description".to_string(),
            Schema::reference("Option<String>"),
        );
        properties.insert("created_at".to_string(), Schema::integer("int64"));
        builder = builder.component_schema(
            "GroupResp",
            Schema::object(
                properties,
                vec![
                    "id".to_string(),
                    "name".to_string(),
                    "description".to_string(),
                    "created_at".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert(
            "groups".to_string(),
            Schema::array(Schema::reference("GroupResp")),
        );
        builder = builder.component_schema(
            "ListGroupsResp",
            Schema::object(properties, vec!["groups".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("group_id".to_string(), Schema::integer("int64"));
        properties.insert("user_id".to_string(), Schema::integer("int64"));
        builder = builder.component_schema(
            "MembershipPathReq",
            Schema::object(
                properties,
                vec!["group_id".to_string(), "user_id".to_string()],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("group_id".to_string(), Schema::integer("int64"));
        properties.insert("user_id".to_string(), Schema::integer("int64"));
        properties.insert("expected_role".to_string(), Schema::string());
        properties.insert("role".to_string(), Schema::string());
        builder = builder.component_schema(
            "UpdateMembershipRoleReq",
            Schema::object(
                properties,
                vec![
                    "group_id".to_string(),
                    "user_id".to_string(),
                    "expected_role".to_string(),
                    "role".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        properties.insert("user_id".to_string(), Schema::integer("int64"));
        properties.insert("group_id".to_string(), Schema::integer("int64"));
        properties.insert("role".to_string(), Schema::string());
        properties.insert("joined_at".to_string(), Schema::integer("int64"));
        builder = builder.component_schema(
            "MembershipResp",
            Schema::object(
                properties,
                vec![
                    "id".to_string(),
                    "user_id".to_string(),
                    "group_id".to_string(),
                    "role".to_string(),
                    "joined_at".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("x-tenant-id".to_string(), Schema::string());
        properties.insert("name".to_string(), Schema::string());
        properties.insert(
            "description".to_string(),
            Schema::reference("Option<String>"),
        );
        builder = builder.component_schema(
            "CreateProjectReq",
            Schema::object(
                properties,
                vec!["x-tenant-id".to_string(), "name".to_string()],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        properties.insert("x-tenant-id".to_string(), Schema::string());
        builder = builder.component_schema(
            "ProjectPathReq",
            Schema::object(
                properties,
                vec!["id".to_string(), "x-tenant-id".to_string()],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("x-tenant-id".to_string(), Schema::string());
        builder = builder.component_schema(
            "ListProjectsReq",
            Schema::object(properties, vec!["x-tenant-id".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        properties.insert("x-tenant-id".to_string(), Schema::string());
        properties.insert("expected_version".to_string(), Schema::integer("int64"));
        properties.insert("name".to_string(), Schema::string());
        properties.insert(
            "description".to_string(),
            Schema::reference("Option<String>"),
        );
        builder = builder.component_schema(
            "UpdateProjectReq",
            Schema::object(
                properties,
                vec![
                    "id".to_string(),
                    "x-tenant-id".to_string(),
                    "expected_version".to_string(),
                    "name".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::integer("int64"));
        properties.insert("tenant_id".to_string(), Schema::string());
        properties.insert("name".to_string(), Schema::string());
        properties.insert(
            "description".to_string(),
            Schema::reference("Option<String>"),
        );
        properties.insert("version".to_string(), Schema::integer("int64"));
        properties.insert("deleted_at".to_string(), Schema::reference("Option<i64>"));
        properties.insert("created_at".to_string(), Schema::integer("int64"));
        properties.insert("updated_at".to_string(), Schema::integer("int64"));
        builder = builder.component_schema(
            "ProjectResp",
            Schema::object(
                properties,
                vec![
                    "id".to_string(),
                    "tenant_id".to_string(),
                    "name".to_string(),
                    "description".to_string(),
                    "version".to_string(),
                    "deleted_at".to_string(),
                    "created_at".to_string(),
                    "updated_at".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert(
            "projects".to_string(),
            Schema::array(Schema::reference("ProjectResp")),
        );
        builder = builder.component_schema(
            "ListProjectsResp",
            Schema::object(properties, vec!["projects".to_string()]),
        );
    }
    {
        let properties = BTreeMap::new();
        builder = builder.component_schema("EmptyReq", Schema::object(properties, vec![]));
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("report".to_string(), Schema::string());
        properties.insert("format".to_string(), Schema::string());
        properties.insert("columns".to_string(), Schema::array(Schema::string()));
        properties.insert(
            "filters".to_string(),
            Schema::object(BTreeMap::new(), Vec::new()),
        );
        properties.insert("from".to_string(), Schema::string());
        properties.insert("to".to_string(), Schema::string());
        properties.insert("timezone".to_string(), Schema::string());
        builder = builder.component_schema(
            "ReportExportRequest",
            Schema::object(properties, vec!["report".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("id".to_string(), Schema::string());
        properties.insert("report".to_string(), Schema::string());
        properties.insert("format".to_string(), Schema::string());
        properties.insert("status".to_string(), Schema::string());
        properties.insert("progress_percent".to_string(), Schema::integer("int32"));
        properties.insert("object_key".to_string(), Schema::string());
        properties.insert("download_url".to_string(), Schema::string());
        properties.insert("expires_at".to_string(), Schema::string());
        properties.insert("error".to_string(), Schema::string());
        properties.insert("tenant_id".to_string(), Schema::string());
        builder = builder.component_schema(
            "ReportExportResource",
            Schema::object(
                properties,
                vec![
                    "id".to_string(),
                    "report".to_string(),
                    "format".to_string(),
                    "status".to_string(),
                    "progress_percent".to_string(),
                    "tenant_id".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("chart".to_string(), Schema::string());
        properties.insert("dimensions".to_string(), Schema::array(Schema::string()));
        properties.insert("measures".to_string(), Schema::array(Schema::string()));
        properties.insert(
            "filters".to_string(),
            Schema::object(BTreeMap::new(), Vec::new()),
        );
        properties.insert("group_by".to_string(), Schema::array(Schema::string()));
        properties.insert("time_bucket".to_string(), Schema::string());
        properties.insert("from".to_string(), Schema::string());
        properties.insert("to".to_string(), Schema::string());
        properties.insert("timezone".to_string(), Schema::string());
        properties.insert("limit".to_string(), Schema::integer("int64"));
        builder = builder.component_schema(
            "ChartQueryRequest",
            Schema::object(properties, vec!["chart".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("timestamp".to_string(), Schema::string());
        properties.insert("value".to_string(), Schema::number("double"));
        properties.insert(
            "labels".to_string(),
            Schema::object(BTreeMap::new(), Vec::new()),
        );
        builder = builder.component_schema(
            "ChartPoint",
            Schema::object(
                properties,
                vec![
                    "timestamp".to_string(),
                    "value".to_string(),
                    "labels".to_string(),
                ],
            ),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("name".to_string(), Schema::string());
        properties.insert(
            "points".to_string(),
            Schema::array(Schema::reference("ChartPoint")),
        );
        builder = builder.component_schema(
            "ChartSeries",
            Schema::object(properties, vec!["name".to_string(), "points".to_string()]),
        );
    }
    {
        let mut properties = BTreeMap::new();
        properties.insert("chart".to_string(), Schema::string());
        properties.insert("dimensions".to_string(), Schema::array(Schema::string()));
        properties.insert("measures".to_string(), Schema::array(Schema::string()));
        properties.insert("time_bucket".to_string(), Schema::string());
        properties.insert("timezone".to_string(), Schema::string());
        properties.insert("scanned_rows".to_string(), Schema::integer("int64"));
        properties.insert("result_rows".to_string(), Schema::integer("int64"));
        properties.insert(
            "series".to_string(),
            Schema::array(Schema::reference("ChartSeries")),
        );
        builder = builder.component_schema(
            "ChartQueryResponse",
            Schema::object(
                properties,
                vec![
                    "chart".to_string(),
                    "dimensions".to_string(),
                    "measures".to_string(),
                    "scanned_rows".to_string(),
                    "result_rows".to_string(),
                    "series".to_string(),
                ],
            ),
        );
    }
    let op = Operation::new("createReportExport")
        .summary("Create an asynchronous report export")
        .tag("roze-ent")
        .request_body("ReportExportRequest")
        .response("200", "Accepted", "ReportExportResource");
    builder.add_operation("/api/v1/reports/exports", HttpMethod::Post, op);
    let op = Operation::new("getReportExport")
        .summary("Get report export status")
        .tag("roze-ent")
        .parameter("id", roze_openapi::ParameterLocation::Path, "String", true)
        .response("200", "OK", "ReportExportResource");
    builder.add_operation("/api/v1/reports/exports/{id}", HttpMethod::Get, op);
    let op = Operation::new("cancelReportExport")
        .summary("Cancel a report export")
        .tag("roze-ent")
        .parameter("id", roze_openapi::ParameterLocation::Path, "String", true)
        .response("200", "OK", "ReportExportResource");
    builder.add_operation("/api/v1/reports/exports/{id}", HttpMethod::Delete, op);
    let op = Operation::new("chartQuery")
        .summary("Run a bounded chart query")
        .tag("roze-ent")
        .request_body("ChartQueryRequest")
        .response("200", "OK", "ChartQueryResponse");
    builder.add_operation("/api/v1/charts/query", HttpMethod::Post, op);
    let op = Operation::new("createUser")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["users:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .request_body("CreateUserReq")
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("UserResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/users", HttpMethod::Post, op);
    let op = Operation::new("getUser")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["users:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("UserResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/users/:id", HttpMethod::Get, op);
    let op = Operation::new("listUsers")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["users:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("ListUsersResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/users", HttpMethod::Get, op);
    let op = Operation::new("deleteUser")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["users:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("DeleteResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/users/:id", HttpMethod::Delete, op);
    let op = Operation::new("createPet")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["pets:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .request_body("CreatePetReq")
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("PetResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/pets", HttpMethod::Post, op);
    let op = Operation::new("getPet")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["pets:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("PetResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/pets/:id", HttpMethod::Get, op);
    let op = Operation::new("listUserPets")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["pets:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("ListPetsResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/users/:id/pets", HttpMethod::Get, op);
    let op = Operation::new("deletePet")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["pets:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("DeleteResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/pets/:id", HttpMethod::Delete, op);
    let op = Operation::new("createGroup")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["groups:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .request_body("CreateGroupReq")
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("GroupResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/groups", HttpMethod::Post, op);
    let op = Operation::new("getGroup")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["groups:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("GroupResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/groups/:id", HttpMethod::Get, op);
    let op = Operation::new("listGroups")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["groups:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("ListGroupsResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/groups", HttpMethod::Get, op);
    let op = Operation::new("addGroupMember")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["groups:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter(
            "group_id",
            roze_openapi::ParameterLocation::Path,
            "i64",
            true,
        )
        .parameter(
            "user_id",
            roze_openapi::ParameterLocation::Path,
            "i64",
            true,
        )
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("MembershipResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation(
        "/api/v1/groups/:group_id/members/:user_id",
        HttpMethod::Post,
        op,
    );
    let op = Operation::new("updateGroupMember")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["groups:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter(
            "group_id",
            roze_openapi::ParameterLocation::Path,
            "i64",
            true,
        )
        .parameter(
            "user_id",
            roze_openapi::ParameterLocation::Path,
            "i64",
            true,
        )
        .request_body("UpdateMembershipRoleReq")
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("MembershipResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation(
        "/api/v1/groups/:group_id/members/:user_id",
        HttpMethod::Patch,
        op,
    );
    let op = Operation::new("removeGroupMember")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["groups:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter(
            "group_id",
            roze_openapi::ParameterLocation::Path,
            "i64",
            true,
        )
        .parameter(
            "user_id",
            roze_openapi::ParameterLocation::Path,
            "i64",
            true,
        )
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("DeleteResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation(
        "/api/v1/groups/:group_id/members/:user_id",
        HttpMethod::Delete,
        op,
    );
    let op = Operation::new("listGroupUsers")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["groups:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("ListUsersResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/groups/:id/users", HttpMethod::Get, op);
    let op = Operation::new("listUserGroups")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["groups:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("ListGroupsResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/users/:id/groups", HttpMethod::Get, op);
    let op = Operation::new("createProject")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["projects:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter(
            "x-tenant-id",
            roze_openapi::ParameterLocation::Header,
            "String",
            true,
        )
        .request_body("CreateProjectReq")
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("ProjectResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/projects", HttpMethod::Post, op);
    let op = Operation::new("getProject")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["projects:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .parameter(
            "x-tenant-id",
            roze_openapi::ParameterLocation::Header,
            "String",
            true,
        )
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("ProjectResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/projects/:id", HttpMethod::Get, op);
    let op = Operation::new("listProjects")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["projects:read"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter(
            "x-tenant-id",
            roze_openapi::ParameterLocation::Header,
            "String",
            true,
        )
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("ListProjectsResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/projects", HttpMethod::Get, op);
    let op = Operation::new("updateProject")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["projects:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .parameter(
            "x-tenant-id",
            roze_openapi::ParameterLocation::Header,
            "String",
            true,
        )
        .request_body("UpdateProjectReq")
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("ProjectResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/projects/:id", HttpMethod::Patch, op);
    let op = Operation::new("deleteProject")
        .tag("roze-ent")
        .extension("x-roze-permissions", serde_json::json!(["projects:write"]))
        .parameter(
            "x-roze-locale",
            roze_openapi::ParameterLocation::Header,
            "String",
            false,
        )
        .parameter("id", roze_openapi::ParameterLocation::Path, "i64", true)
        .parameter(
            "x-tenant-id",
            roze_openapi::ParameterLocation::Header,
            "String",
            true,
        )
        .response_with_schema("200", "OK", "application/json", {
            let mut properties = BTreeMap::new();
            properties.insert("code".to_string(), Schema::integer("int32"));
            properties.insert("msg".to_string(), Schema::string());
            properties.insert("data".to_string(), Schema::reference("DeleteResp"));
            Schema::object(
                properties,
                vec!["code".to_string(), "msg".to_string(), "data".to_string()],
            )
        });
    builder.add_operation("/api/v1/projects/:id", HttpMethod::Delete, op);
    roze_openapi::to_json_value(&builder.finish())
}
