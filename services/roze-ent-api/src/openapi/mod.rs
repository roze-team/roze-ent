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
    roze_openapi::to_json_value(&builder.finish())
}
