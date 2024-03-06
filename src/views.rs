pub async fn employees(
    opts: Option<Query<QueryOptions>>,
    Extension(templates): Extension<Templates>,
    State(db): State<DB>,
) -> impl IntoResponse {
    let employees = db.lock().await;

    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // list not onboarded employees
    let employees: Vec<Employee> = employees
        .clone()
        .into_iter()
        .skip(offset)
        .take(limit)
        .filter(|employee: &Employee| employee.onboarded == Some(false))
        .collect();

    let json_response = EmployeeListResponse {
        status: "success".to_string(),
        results: employees.len(),
        todos: employees.clone(),
    };

    debug!("{json_response:?}");

    let mut context = Context::new();
    context.insert("employees", &employees.to_owned());

    Html(templates.render("employees", &context).unwrap())
}

pub async fn employee(
    Path(id): Path<Uuid>,
    Extension(templates): Extension<Templates>,
    State(db): State<DB>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let vec = db.lock().await;

    if let Some(employee) = vec
        .iter()
        .find(|employee: &&Employee| employee.id == Some(id))
    {
        let json_response = SimpleEmployeeResponse {
            status: "success".to_string(),
            data: EmployeeData {
                employee: employee.clone(),
            },
        };
        debug!("{json_response:?}");
        let mut context = Context::new();
        context.insert("first_name", &employee.first_name);
        context.insert("last_name", &employee.last_name);
        //return Ok((StatusCode::OK, Json(json_response)));
        return Ok(Html(templates.render("employee", &context).unwrap()));
    }

    let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("Employee with ID: {} not found", id)
    });
    debug!("{error_response:?}");
    Err((StatusCode::NOT_FOUND, Json(error_response)))
}
